// Qualium Quantum Browser v1 — Shell Controller
function logDebug(msg) {
  try {
    const file = Cc["@mozilla.org/file/local;1"].createInstance(Ci.nsIFile);
    file.initWithPath("C:\\Users\\mndab\\AppData\\Local\\Temp\\qualium_shell.log");
    const foStream = Cc["@mozilla.org/network/file-output-stream;1"].createInstance(Ci.nsIFileOutputStream);
    foStream.init(file, 0x02 | 0x08 | 0x10, 0o666, 0);
    const data = "[" + new Date().toISOString() + "] " + msg + "\n";
    foStream.write(data, data.length);
    foStream.close();
  } catch(e) {}
}

logDebug("qualium_shell.js evaluated");

window.addEventListener("DOMContentLoaded", () => {
  logDebug("DOMContentLoaded fired in qualium_browser.xhtml");
  const browser = document.getElementById("content-browser");
  logDebug("content-browser element found: " + (!!browser));
  if (browser) {
    logDebug("browser tagName=" + browser.tagName + ", type=" + browser.getAttribute("type") + ", hasLoadURI=" + (typeof browser.loadURI === "function"));
  }

  const urlInput = document.getElementById("url-input");
  const tabTitle = document.getElementById("tab-1-title");
  const btnBack = document.getElementById("btn-back");
  const btnForward = document.getElementById("btn-forward");
  const btnReload = document.getElementById("btn-reload");
  const btnQPrivacy = document.getElementById("btn-q-privacy");

  function navigateTo(input) {
    logDebug("navigateTo called with: " + input);
    if (!input || !browser) return;
    let url = input.trim();
    if (!url) return;

    if (url === "qualium://newtab") {
      url = "chrome://qualium/content/newtab.xhtml";
    } else if (url === "qualium://privacy" || url === "qualium://security") {
      url = "chrome://qualium/content/dashboard.xhtml";
    } else if (url.startsWith("qualium://")) {
      url = "chrome://qualium/content/" + url.replace("qualium://", "") + ".xhtml";
    } else if (/^https?:\/\//i.test(url)) {
      // already full URL
    } else if (/^([a-z0-9-]+\.)+[a-z]{2,}(\/.*)?$/i.test(url) && !url.includes(" ")) {
      url = "https://" + url;
    } else {
      url = "https://www.google.com/search?q=" + encodeURIComponent(url);
    }

    logDebug("Resolved URL: " + url);

    try {
      if (typeof browser.loadURI === "function") {
        try {
          const secMan = Services.scriptSecurityManager;
          browser.loadURI(Services.io.newURI(url), {
            triggeringPrincipal: secMan.getSystemPrincipal(),
          });
          logDebug("loadURI succeeded with URI object");
        } catch (e1) {
          logDebug("loadURI with URI object failed: " + e1 + ", trying string loadURI");
          browser.loadURI(url, { triggeringPrincipal: Services.scriptSecurityManager.getSystemPrincipal() });
          logDebug("loadURI with string succeeded");
        }
      } else {
        logDebug("Setting src attribute directly");
        browser.setAttribute("src", url);
      }
    } catch (err) {
      logDebug("Navigation error: " + err);
      browser.setAttribute("src", url);
    }
  }

  window.qualiumNavigate = navigateTo;

  if (urlInput) {
    urlInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        navigateTo(urlInput.value);
      }
    });
  }

  if (btnBack) {
    btnBack.addEventListener("click", () => {
      try { browser.goBack(); } catch(e) {}
    });
  }

  if (btnForward) {
    btnForward.addEventListener("click", () => {
      try { browser.goForward(); } catch(e) {}
    });
  }

  if (btnReload) {
    btnReload.addEventListener("click", () => {
      try { browser.reload(); } catch(e) {}
    });
  }

  if (btnQPrivacy) {
    btnQPrivacy.addEventListener("click", () => {
      navigateTo("qualium://privacy");
    });
  }

  // Auto navigate to newtab
  setTimeout(() => {
    navigateTo("qualium://newtab");
  }, 100);
});
