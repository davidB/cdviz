import "./style.css";

function selectHeroImage() {
  const randomInt = Math.floor(Math.random() * 8);
  const heroImage = document.getElementById("hero-image-img");
  heroImage.src = `./assets/illustrations/hero-${randomInt}.webp`;
}

selectHeroImage();
