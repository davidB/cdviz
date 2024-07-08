-- find all subject
SELECT DISTINCT "subject" FROM "cdevents_lake";

-- find all predicate for a subject
SELECT DISTINCT "predicate" FROM "cdevents_lake" WHERE subject IN ($subjects)

-- 
select payload -> 'subject' -> 'content' -> 'artifactId' as artifact from cdevents_lake
where subject = 'service'
and  predicate = 'deployed'
-- and payload -> 'subject' -> 'content' -> 'environment' ->> 'id' IN ('%')
-- and payload -> 'subject' -> 'content' ->> 'artfactId' IN ('%')
;

-- 'pkg:oci/myapp@sha256%3A0b31b1c02ff458ad9b7b81cbdf8f028bd54699fa151f221d1e8de6817db93427'
select alias, description, token from ts_debug('pkg:oci/myapp@sha256%3A0b31b1c02ff458ad9b7b81cbdf8f028bd54699fa151f221d1e8de6817db93427')
;

select regexp_match (
 'pkg:oci/myapp@sha256%3A0b31b1c02ff458ad9b7b81cbdf8f028bd54699fa151f221d1e8de6817db93427',
  'pkg:(\w*)/(.*)@(.*)(\?.*)?(#.*)?' 
)
;

drop function extract_purl_fields;
DROP type purl_fields;

create TYPE purl_fields AS ("type" varchar(32), "name" varchar(256) , "version" varchar(128), "qualifier" varchar(256), "subpath" varchar(256));

CREATE OR REPLACE FUNCTION extract_purl_fields(purl text) 
RETURNS purl_fields 
AS 
$$
DECLARE
  result_record public.purl_fields;

BEGIN
  select  m[1], m[2], m[3], m[4], m[5]
  INTO result_record.type, result_record.name, result_record.version, result_record.qualifier, result_record.subpath
  from regexp_match (purl, 'pkg:(\w*)/(.*)@(.*)(\?.*)?(#.*)?') as m
  ;
  RETURN result_record;
END
$$ LANGUAGE plpgsql;

select extract_purl_fields('pkg:oci/myapp@sha256%3A0b31b1c02ff458ad9b7b81cbdf8f028bd54699fa151f221d1e8de6817db93427')

select distinct a.name from cdevents_lake as c, extract_purl_fields(c.payload -> 'subject' -> 'content' ->> 'artifactId') as a;

select extract_purl_fields(payload -> 'subject' -> 'content' ->> 'artifactId') as artifact from cdevents_lake
where subject = 'service'
and  predicate = 'deployed'
and payload -> 'subject' -> 'content' -> 'environment' ->> 'id' similar to  '(%)'
and payload -> 'subject' -> 'content' ->> 'artfactId' similar to '/(%)?'
;

--TODO define a procedural function to insert event (how to validate json)
--TODO list query (natural + sql) about what we want
-- - what is the deployed version per environment for application X at time t ?
-- - what is the history of an application (optional filter version, environment)?
-- - what is the status of application's version ?
-- - what is deployed in environment (pattern) ?
-- - how many time for a version between 2 environment?
-- - how many time for a version between build/publish/first deploy/last deploy?
-- order version by timestamp (on ???)
--TODO create readonly view per subject with some extraction ?
-- compare (explain) query over view vs query direct on lake
--TODO create the role to view data
--TODO create the role to push data (via the procedure call)