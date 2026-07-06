// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

document.addEventListener("DOMContentLoaded", () => {
  const toggle = document.getElementById("navToggle");
  const nav = document.getElementById("siteNav");

  if (!toggle || !nav) return;

  const closeNav = () => {
    nav.classList.remove("is-open");
    toggle.setAttribute("aria-expanded", "false");
  };

  toggle.addEventListener("click", (e) => {
    e.stopPropagation();
    const isOpen = nav.classList.toggle("is-open");
    toggle.setAttribute("aria-expanded", String(isOpen));
  });

  nav.querySelectorAll("a").forEach((link) => {
    link.addEventListener("click", closeNav);
  });

  document.addEventListener("click", (e) => {
    if (!nav.contains(e.target) && !toggle.contains(e.target)) {
      closeNav();
    }
  });
});
