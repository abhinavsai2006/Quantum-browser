// Qualium Privacy Center Navigation & Telemetry
document.addEventListener("DOMContentLoaded", () => {
  const navItems = document.querySelectorAll(".nav-item");
  const panes = document.querySelectorAll(".content-pane");

  navItems.forEach(item => {
    item.addEventListener("click", () => {
      navItems.forEach(i => i.classList.remove("active"));
      panes.forEach(p => p.classList.remove("active"));

      item.classList.add("active");
      const targetId = "pane-" + item.getAttribute("data-target");
      const targetPane = document.getElementById(targetId);
      if (targetPane) {
        targetPane.classList.add("active");
      }
    });
  });
});
