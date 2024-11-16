import "./style.css";

const heroImage = document.getElementById("hero-image-img");
const randomInt = Math.floor(Math.random() * 8);
heroImage.src = `./assets/illustrations/hero-${randomInt}.webp`;
console.log(randomInt);