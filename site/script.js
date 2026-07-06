async function loadLatestVersion() {
  const versionLink = document.getElementById("latestVersionLink");
  if (!versionLink) return;

  const fallbackUrl = "https://github.com/SinTan1729/chhoto-url/releases";

  try {
    const response = await fetch(
      "https://api.github.com/repos/SinTan1729/chhoto-url/releases/latest",
      {
        headers: {
          Accept: "application/vnd.github+json",
        },
      },
    );

    if (!response.ok) {
      throw new Error(`GitHub API returned ${response.status}`);
    }

    const release = await response.json();
    const version = release.tag_name || release.name || "Unavailable";

    versionLink.textContent = "v" + version;
    versionLink.href = release.html_url || fallbackUrl;
  } catch (error) {
    versionLink.textContent = "View releases";
    versionLink.href = fallbackUrl;
    console.error("Failed to load latest version:", error);
  }
}

document.addEventListener("DOMContentLoaded", loadLatestVersion);

document.addEventListener("DOMContentLoaded", () => {
  loadLatestVersion.await;
  const toggle = document.getElementById("navToggle");
  const nav = document.getElementById("siteNav");

  if (!toggle || !nav) return;

  toggle.addEventListener("click", () => {
    const isOpen = nav.classList.toggle("is-open");
    toggle.setAttribute("aria-expanded", String(isOpen));
  });

  nav.querySelectorAll("a").forEach((link) => {
    link.addEventListener("click", () => {
      nav.classList.remove("is-open");
      toggle.setAttribute("aria-expanded", "false");
    });
  });
});
