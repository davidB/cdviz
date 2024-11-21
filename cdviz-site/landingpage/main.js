import "./style.css";

function selectHeroImage() {
  const randomInt = Math.floor(Math.random() * 8);
  const heroImage = document.getElementById("hero-image-img");
  heroImage.src = `./assets/illustrations/hero-${randomInt}.webp`;
}

function initColorMode() {
  // It's best to inline this in `head` to avoid FOUC (flash of unstyled content) when changing pages or themes
  if (
    localStorage.getItem("color-theme") === "dark" ||
    (!("color-theme" in localStorage) &&
      window.matchMedia("(prefers-color-scheme: dark)").matches)
  ) {
    document.documentElement.classList.add("dark");
  } else {
    document.documentElement.classList.remove("dark");
  }
}
initColorMode();
selectHeroImage();
