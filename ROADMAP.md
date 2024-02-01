# Roadmap

## 0.1.0

- [x] bootstrap db
- [x] bootstrap `cdviz-collector`: store on db
- [ ] bootstrap `cdviz-sensors`: push events from folder to `cdviz-collector` via http
- [ ] bootstrap `cdviz-demo`
- [ ] bootstrap grafana dashboards
- [x] helm charts to deploy (on local)
- [ ] publish workflow, host runtime packages on github (container images and chart)

## 0.2.0

- [ ] autocreate the read-only user on db (how?)
- [ ] validate [cdevents] (rust library) on `cdviz-collector` and `cdviz-sensors`
- [ ] connects to NATS (`cdviz-collector` and `cdviz-sensors`)
- [ ] bootstrap the demo stack: NATS, testkube

## 0.3.0

- [ ] document API
- [ ] document DB schema
- [ ] document for contribution
- [ ] document the demo
- [ ] cdviz-sensors start to watch K8S events for "deployment" (remove & update)

## ?.?.?

- [ ] Do everything 😅

## Ideas & Maybe

- a simple ruler to trigger for events likes:
  - on deploy of version X of service, send a remove of previous version (same service, package, environment)
- collect SBOM, and connect it to the events to enhance info, lifecycle of components of deployed artifacts
- deduce SBOM (via rules, ...)
