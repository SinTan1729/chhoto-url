// SPDX-FileCopyrightText: 2023-2026 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

// Application state
let VERSION = null;
let SITE_URL = "-";
let CONFIG = null;
let SUBDIR = null;
let ADMIN = null;
let LOCAL_DATA = [];
let CUR_PAGE = 0;
let FILTER = null;

// Flags
let PROCESSING_PAGE_TRANSITION = true;

// Buttons
// https://svgicons.com/icon/10648/copy-outline
SVG_COPY_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M9 3.25A5.75 5.75 0 0 0 3.25 9v7.107a.75.75 0 0 0 1.5 0V9A4.25 4.25 0 0 1 9 4.75h7.013a.75.75 0 0 0 0-1.5z"/><path fill="currentColor" fill-rule="evenodd" d="M18.403 6.793a44.372 44.372 0 0 0-9.806 0a2.011 2.011 0 0 0-1.774 1.76a42.581 42.581 0 0 0 0 9.894a2.01 2.01 0 0 0 1.774 1.76c3.241.362 6.565.362 9.806 0a2.01 2.01 0 0 0 1.774-1.76a42.579 42.579 0 0 0 0-9.894a2.011 2.011 0 0 0-1.774-1.76M8.764 8.284c3.13-.35 6.342-.35 9.472 0a.51.51 0 0 1 .45.444a40.95 40.95 0 0 1 0 9.544a.51.51 0 0 1-.45.444c-3.13.35-6.342.35-9.472 0a.511.511 0 0 1-.45-.444a40.95 40.95 0 0 1 0-9.544a.511.511 0 0 1 .45-.444" clip-rule="evenodd"/></svg>`;
// https://svgicons.com/icon/1207/qrcode-outlined
SVG_QR_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 1024 1024"><path fill="currentColor" d="M468 128H160c-17.7 0-32 14.3-32 32v308c0 4.4 3.6 8 8 8h332c4.4 0 8-3.6 8-8V136c0-4.4-3.6-8-8-8m-56 284H192V192h220zm-138-74h56c4.4 0 8-3.6 8-8v-56c0-4.4-3.6-8-8-8h-56c-4.4 0-8 3.6-8 8v56c0 4.4 3.6 8 8 8m194 210H136c-4.4 0-8 3.6-8 8v308c0 17.7 14.3 32 32 32h308c4.4 0 8-3.6 8-8V556c0-4.4-3.6-8-8-8m-56 284H192V612h220zm-138-74h56c4.4 0 8-3.6 8-8v-56c0-4.4-3.6-8-8-8h-56c-4.4 0-8 3.6-8 8v56c0 4.4 3.6 8 8 8m590-630H556c-4.4 0-8 3.6-8 8v332c0 4.4 3.6 8 8 8h332c4.4 0 8-3.6 8-8V160c0-17.7-14.3-32-32-32m-32 284H612V192h220zm-138-74h56c4.4 0 8-3.6 8-8v-56c0-4.4-3.6-8-8-8h-56c-4.4 0-8 3.6-8 8v56c0 4.4 3.6 8 8 8m194 210h-48c-4.4 0-8 3.6-8 8v134h-78V556c0-4.4-3.6-8-8-8H556c-4.4 0-8 3.6-8 8v332c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8V644h78v102c0 4.4 3.6 8 8 8h190c4.4 0 8-3.6 8-8V556c0-4.4-3.6-8-8-8M746 832h-48c-4.4 0-8 3.6-8 8v48c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8v-48c0-4.4-3.6-8-8-8m142 0h-48c-4.4 0-8 3.6-8 8v48c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8v-48c0-4.4-3.6-8-8-8"/></svg>`;
// https://svgicons.com/icon/10674/edit-outline
SVG_EDIT_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M21.455 5.416a.75.75 0 0 1-.096.943l-9.193 9.192a.75.75 0 0 1-.34.195l-3.829 1a.75.75 0 0 1-.915-.915l1-3.828a.778.778 0 0 1 .161-.312L17.47 2.47a.75.75 0 0 1 1.06 0l2.829 2.828a.756.756 0 0 1 .096.118m-1.687.412L18 4.061l-8.518 8.518l-.625 2.393l2.393-.625z" clip-rule="evenodd"/><path fill="currentColor" d="M19.641 17.16a44.4 44.4 0 0 0 .261-7.04a.403.403 0 0 1 .117-.3l.984-.984a.198.198 0 0 1 .338.127a45.91 45.91 0 0 1-.21 8.372c-.236 2.022-1.86 3.607-3.873 3.832a47.77 47.77 0 0 1-10.516 0c-2.012-.225-3.637-1.81-3.873-3.832a45.922 45.922 0 0 1 0-10.67c.236-2.022 1.86-3.607 3.873-3.832a47.75 47.75 0 0 1 7.989-.213a.2.2 0 0 1 .128.34l-.993.992a.402.402 0 0 1-.297.117a46.164 46.164 0 0 0-6.66.255a2.89 2.89 0 0 0-2.55 2.516a44.421 44.421 0 0 0 0 10.32a2.89 2.89 0 0 0 2.55 2.516c3.355.375 6.827.375 10.183 0a2.89 2.89 0 0 0 2.55-2.516"/></svg>`;
// https://svgicons.com/icon/10955/trash-solid
SVG_DELETE_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M10 2.25a.75.75 0 0 0-.75.75v.75H5a.75.75 0 0 0 0 1.5h14a.75.75 0 0 0 0-1.5h-4.25V3a.75.75 0 0 0-.75-.75zM13.06 15l1.47 1.47a.75.75 0 1 1-1.06 1.06L12 16.06l-1.47 1.47a.75.75 0 1 1-1.06-1.06L10.94 15l-1.47-1.47a.75.75 0 1 1 1.06-1.06L12 13.94l1.47-1.47a.75.75 0 1 1 1.06 1.06z"/><path fill="currentColor" fill-rule="evenodd" d="M5.991 7.917a.75.75 0 0 1 .746-.667h10.526a.75.75 0 0 1 .746.667l.2 1.802c.363 3.265.363 6.56 0 9.826l-.02.177a2.853 2.853 0 0 1-2.44 2.51a27.04 27.04 0 0 1-7.498 0a2.853 2.853 0 0 1-2.44-2.51l-.02-.177a44.489 44.489 0 0 1 0-9.826zm1.417.833l-.126 1.134a42.99 42.99 0 0 0 0 9.495l.02.177a1.353 1.353 0 0 0 1.157 1.191c2.35.329 4.733.329 7.082 0a1.353 1.353 0 0 0 1.157-1.19l.02-.178c.35-3.155.35-6.34 0-9.495l-.126-1.134z" clip-rule="evenodd"/></svg>`;
// https://svgicons.com/icon/10689/eye-solid
SVG_OPEN_EYE = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 9.75a2.25 2.25 0 1 0 0 4.5a2.25 2.25 0 0 0 0-4.5"/><path fill="currentColor" fill-rule="evenodd" d="M12 5.5c-2.618 0-4.972 1.051-6.668 2.353c-.85.652-1.547 1.376-2.036 2.08c-.48.692-.796 1.418-.796 2.067c0 .649.317 1.375.796 2.066c.49.705 1.186 1.429 2.036 2.08C7.028 17.45 9.382 18.5 12 18.5c2.618 0 4.972-1.051 6.668-2.353c.85-.652 1.547-1.376 2.035-2.08c.48-.692.797-1.418.797-2.067c0-.649-.317-1.375-.797-2.066c-.488-.705-1.185-1.429-2.035-2.08C16.972 6.55 14.618 5.5 12 5.5M8.25 12a3.75 3.75 0 1 1 7.5 0a3.75 3.75 0 0 1-7.5 0" clip-rule="evenodd"/></svg>`;
// https://svgicons.com/icon/10687/eye-closed-solid
SVG_CLOSED_EYE = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M20.53 4.53a.75.75 0 0 0-1.06-1.06l-16 16a.75.75 0 1 0 1.06 1.06l3.035-3.035C8.883 18.103 10.392 18.5 12 18.5c2.618 0 4.972-1.051 6.668-2.353c.85-.652 1.547-1.376 2.035-2.08c.48-.692.797-1.418.797-2.067c0-.649-.317-1.375-.797-2.066c-.488-.705-1.185-1.429-2.035-2.08c-.27-.208-.558-.41-.86-.601zm-5.4 5.402l-1.1 1.098a2.25 2.25 0 0 1-3 3l-1.1 1.1a3.75 3.75 0 0 0 5.197-5.197" clip-rule="evenodd"/><path fill="currentColor" d="M12.67 8.31a.26.26 0 0 0 .23-.07l1.95-1.95a.243.243 0 0 0-.104-.407A10.214 10.214 0 0 0 12 5.5c-2.618 0-4.972 1.051-6.668 2.353c-.85.652-1.547 1.376-2.036 2.08c-.48.692-.796 1.418-.796 2.067c0 .649.317 1.375.796 2.066a9.287 9.287 0 0 0 1.672 1.79a.246.246 0 0 0 .332-.017l2.94-2.94a.26.26 0 0 0 .07-.23a3.75 3.75 0 0 1 4.36-4.36"/></svg>`;
// https://svgicons.com/icon/10926/skip-prev-outline
SVG_PREV_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M6.75 7a.75.75 0 0 0-1.5 0v10a.75.75 0 0 0 1.5 0z"/><path fill="currentColor" fill-rule="evenodd" d="M9.393 13.253a1.584 1.584 0 0 1 0-2.505a25.76 25.76 0 0 1 7.143-3.902l.466-.165c1.023-.364 2.1.329 2.238 1.381c.34 2.59.34 5.286 0 7.876c-.138 1.052-1.215 1.745-2.238 1.381l-.466-.165a25.758 25.758 0 0 1-7.143-3.902m.918-1.32a.084.084 0 0 0 0 .133a24.257 24.257 0 0 0 6.727 3.674l.466.166c.1.035.232-.033.249-.163c.322-2.46.322-5.025 0-7.486a.194.194 0 0 0-.25-.163l-.465.166c-2.423.86-4.694 2.1-6.727 3.674" clip-rule="evenodd"/></svg>`;
// https://svgicons.com/icon/10924/skip-next-outline
SVG_NEXT_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M14.607 10.748c.82.634.82 1.87 0 2.505a25.758 25.758 0 0 1-7.143 3.9l-.466.166c-1.023.364-2.1-.329-2.238-1.381c-.34-2.59-.34-5.286 0-7.876c.138-1.052 1.215-1.745 2.238-1.381l.466.165a25.76 25.76 0 0 1 7.143 3.902m-.918 1.318a.084.084 0 0 0 0-.132A24.257 24.257 0 0 0 6.962 8.26l-.466-.166a.194.194 0 0 0-.249.163a29.063 29.063 0 0 0 0 7.486c.017.13.15.198.25.163l.465-.166c2.423-.86 4.694-2.1 6.727-3.674M18 6.25a.75.75 0 0 1 .75.75v10a.75.75 0 0 1-1.5 0V7a.75.75 0 0 1 .75-.75" clip-rule="evenodd"/></svg>`;
// https://svgicons.com/icon/10769/info-rect-outline
SVG_INFO_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M12 10.75a.75.75 0 0 1 .75.75v5a.75.75 0 1 1-1.5 0v-5a.75.75 0 0 1 .75-.75M12 9a1 1 0 1 0 0-2a1 1 0 0 0 0 2"/><path fill="currentColor" fill-rule="evenodd" d="M7.317 3.769a42.5 42.5 0 0 1 9.366 0c1.827.204 3.302 1.642 3.516 3.48c.37 3.156.37 6.346 0 9.503c-.215 1.836-1.69 3.275-3.516 3.48a42.5 42.5 0 0 1-9.366 0c-1.827-.205-3.302-1.644-3.516-3.48a41 41 0 0 1 0-9.504c.214-1.837 1.69-3.275 3.516-3.48m9.2 1.49a41 41 0 0 0-9.034 0A2.486 2.486 0 0 0 5.29 7.423a39.4 39.4 0 0 0 0 9.154a2.486 2.486 0 0 0 2.193 2.163c2.977.333 6.057.333 9.034 0a2.486 2.486 0 0 0 2.192-2.163a39.4 39.4 0 0 0 0-9.154a2.486 2.486 0 0 0-2.192-2.164" clip-rule="evenodd"/></svg>`;

// in miliseconds
const UNITS = {
  year: 31536000000,
  month: 2592000000,
  day: 86400000,
  hour: 3600000,
  minute: 60000,
  second: 1000,
};

const loadCachedState = () => {
  try {
    const cachedAdmin = sessionStorage.getItem("admin");
    const cachedConfig = sessionStorage.getItem("config");

    if (cachedAdmin !== null) {
      ADMIN = cachedAdmin === "true";
    }
    if (cachedConfig !== null) {
      CONFIG = JSON.parse(cachedConfig);
    }
  } catch (err) {
    clearCachedState();
  }
};

const clearCachedState = () => {
  ADMIN = null;
  CONFIG = null;
  sessionStorage.removeItem("admin");
  sessionStorage.removeItem("config");
};

const cacheAdmin = (admin) => {
  ADMIN = admin;
  sessionStorage.setItem("admin", String(admin));
};

const cacheConfig = (config) => {
  CONFIG = config;
  sessionStorage.setItem("config", JSON.stringify(config));
};

const prepSubdir = (link) => {
  if (!SUBDIR) {
    const thisPage = new URL(window.location.href);
    SUBDIR = thisPage.pathname.replace(/\/admin\/manage\/$/, "/");
  }
  return (SUBDIR + link).replace("//", "/");
};

const hasAllowedScheme = (url) => {
  const allowedSchemes = ["http:", "https:", "ftp:", "magnet:"];
  try {
    return allowedSchemes.includes(new URL(url).protocol);
  } catch {
    return false;
  }
};

const getConfig = async () => {
  if (ADMIN == null) {
    return;
  }
  if (CONFIG == null) {
    const res = await fetch(prepSubdir("/api/getconfig"), {
      cache: "no-cache",
    });
    if (!res.ok) {
      console.log("Error while fetching config.");
      clearCachedState();
      return;
    }

    const config = await res.json();
    if (config.error == null) {
      cacheConfig(config);
    } else {
      return;
    }
  }

  VERSION = CONFIG.version;
  if (CONFIG.site_url == null) {
    SITE_URL = window.location.host;
  } else {
    SITE_URL = CONFIG.site_url.replace(/^"/, "").replace(/"$/, "");
    const url = new URL(SITE_URL);
    SITE_URL = SITE_URL.replace(url.hostname, punycode.toUnicode(url.hostname));
    SITE_URL = SITE_URL.replace(url.pathname, punycode.toUnicode(url.pathname));
    SITE_URL = SITE_URL.replace(/\/$/, "");
  }

  if (CONFIG.frontend_page_size == null) {
    CONFIG.frontend_page_size = 10;
  }

  if (!hasAllowedScheme(SITE_URL)) {
    SITE_URL = window.location.protocol + "//" + SITE_URL;
  }

  VERSION = CONFIG.version;
};

const showVersion = () => {
  const link = document.getElementById("version-number");
  if (VERSION) {
    link.getElementsByTagName("span")[0].innerText = "v" + VERSION;
    if (VERSION.includes("-dev+")) {
      link.href =
        "https://github.com/SinTan1729/chhoto-url/commits/" +
        VERSION.split("+")[1];
    } else {
      link.href =
        "https://github.com/SinTan1729/chhoto-url/releases/tag/" + VERSION;
    }
    link.hidden = false;
  } else {
    link.hidden = true;
  }
};

const showLogin = () => {
  document.getElementById("version-number").hidden = true;
  document.getElementById("admin-button").hidden = true;
  document.getElementById("container").style.filter = "blur(2px)";
  document.getElementById("login-dialog").showModal();
  document.getElementById("password").focus();
};

const refreshData = async () => {
  try {
    const loading_text = document.getElementById("loading-text");
    const admin_button = document.getElementById("admin-button");

    const getPullParams = () => {
      const params = new URLSearchParams();

      if (LOCAL_DATA.length == 0) {
        params.append("page_size", 2 * CONFIG.frontend_page_size);
      } else {
        if (LOCAL_DATA.length <= CUR_PAGE * CONFIG.frontend_page_size) {
          console.log("Reached the end of URLs.");
          return null;
        }
        params.append("page_size", CONFIG.frontend_page_size);
        params.append("page_after", LOCAL_DATA.at(-1)["shortlink"]);
      }

      return params;
    };

    if (ADMIN === null && CONFIG == null) {
      loadCachedState();
    }

    if (ADMIN === true && CONFIG != null) {
      try {
        await getConfig();
        showVersion();
        const admin_button = document.getElementById("admin-button");
        admin_button.getElementsByTagName("span")[0].innerText = "logout";
        admin_button.hidden = false;

        const params = getPullParams();
        if (params == null) {
          return;
        }

        if (LOCAL_DATA.length != 0) {
          displayData();
        }

        const data = await pullData(params);
        LOCAL_DATA.push(...data.reverse());

        if (CUR_PAGE == 0) {
          displayData();
        }
        managePageControls();
        return;
      } catch (err) {
        console.log("/api/all failed, clearing cache and falling back.", err);
        clearCachedState();
      }
    }

    if (ADMIN !== true) {
      const res = await fetch(prepSubdir("/api/whoami"), { cache: "no-cache" });
      if (res.status !== 200) {
        throw Error("There was an issue getting user role.");
      }

      const role = await res.text();
      switch (role) {
        case "nobody":
          clearCachedState();
          showLogin();
          return;

        case "public":
          cacheAdmin(false);
          await getConfig();

          loading_text.innerHTML = "Using public mode.";
          const expiry = parseInt(CONFIG.public_mode_expiry_delay);
          if (expiry > 0) {
            loading_text.innerHTML +=
              " Unless chosen a shorter expiry time, submitted links will automatically expire ";
            const time = new Date();
            time.setSeconds(time.getSeconds() + expiry);
            loading_text.innerHTML += formatRelativeTime(time) + ".";
          }

          admin_button.getElementsByTagName("span")[0].innerText = "login";
          admin_button.hidden = false;
          updateInputBox();
          break;

        case "admin":
          cacheAdmin(true);
          await getConfig();
          break;

        default:
          throw Error("Got undefined user role.");
      }
    }

    showVersion();

    if (ADMIN === true) {
      const params = getPullParams();
      if (params == null) {
        return;
      }

      if (LOCAL_DATA.length != 0) {
        displayData();
      }

      let data;
      try {
        data = await pullData(params);
      } catch (err) {
        clearCachedState();
        throw err;
      }

      LOCAL_DATA.push(...data.reverse());

      if (CUR_PAGE == 0) {
        displayData();
      }
      managePageControls();
    } else {
      document.getElementById("table-box").hidden = true;
      loading_text.hidden = false;
      document.getElementById("url-table").innerHTML = "";
    }
  } catch (err) {
    console.log(err);
    if (!alert("Something went wrong! Click OK to refresh page.")) {
      window.location.reload();
    }
  }
};

const pullData = async (params) => {
  if (FILTER != null) {
    params.append("filter", FILTER);
  }
  const res = await fetch(prepSubdir(`/api/all?${params}`), {
    cache: "no-cache",
  });
  if (res.status == 200) {
    const data = await res.json();
    return data;
  } else {
    throw Error("There was an error getting data.");
  }
};

const gotoPrevPage = () => {
  if (PROCESSING_PAGE_TRANSITION) {
    return;
  }
  PROCESSING_PAGE_TRANSITION = true;
  if (CUR_PAGE > 0) {
    CUR_PAGE -= 1;
  }
  displayData();
  managePageControls();
};

const gotoNextPage = () => {
  if (PROCESSING_PAGE_TRANSITION) {
    return;
  }
  PROCESSING_PAGE_TRANSITION = true;
  CUR_PAGE += 1;
  if (LOCAL_DATA.length <= (CUR_PAGE + 1) * CONFIG.frontend_page_size) {
    refreshData();
  } else {
    displayData();
    managePageControls();
  }
};

const updateInputBox = () => {
  if (CONFIG.allow_capital_letters) {
    const input_box = document.getElementById("shortUrl");
    input_box.pattern = "[A-Za-z0-9\\\-_]+";
    input_box.title = "Only A-Z, a-z, 0-9, - and _ are allowed";
    input_box.placeholder = "Only A-Z, a-z, 0-9, - and _ are allowed";
  }
};

const refreshWithFilter = () => {
  const filterInput = document.getElementById("filterText");
  const filter = filterInput.value;

  filterInput.setCustomValidity("");
  const oldFilter = FILTER;
  if (filter == "") {
    FILTER = null;
  } else {
    FILTER = filter;
  }
  if (FILTER != oldFilter) {
    LOCAL_DATA = [];
    refreshData();
  }
};

const displayData = () => {
  if (CUR_PAGE < 0) {
    console.log("Trying to access negative numbered page.");
    return;
  }
  const data = LOCAL_DATA.slice(
    CUR_PAGE * CONFIG.frontend_page_size,
    (CUR_PAGE + 1) * CONFIG.frontend_page_size,
  );
  showVersion();
  const admin_button = document.getElementById("admin-button");
  admin_button.getElementsByTagName("span")[0].innerText = "logout";
  admin_button.hidden = false;
  updateInputBox();

  const table_box = document.getElementById("table-box");
  const loading_text = document.getElementById("loading-text");
  const table = document.getElementById("url-table");

  if (data.length === 0 && FILTER == null) {
    table_box.hidden = true;
    loading_text.innerHTML = "No active links.";
    loading_text.hidden = false;
  } else {
    loading_text.hidden = true;
    table_box.hidden = false;
    table.innerHTML = "";
    for (const [i, row] of data.entries()) {
      table.appendChild(TR(CUR_PAGE * CONFIG.frontend_page_size + i + 1, row));
    }
    setTimeout(refreshExpiryTimes, 1000);
  }
};

const managePageControls = () => {
  const on_first_page = CUR_PAGE == 0;
  const on_last_page =
    LOCAL_DATA.length <= (CUR_PAGE + 1) * CONFIG.frontend_page_size;

  document.getElementById("prevPageBtn").disabled = on_first_page;
  document.getElementById("nextPageBtn").disabled = on_last_page;
  document.getElementById("pageControls").hidden =
    on_first_page && on_last_page;
  PROCESSING_PAGE_TRANSITION = false;
};

const showAlert = (text, col) => {
  const alertBox = document.getElementById("alert-box");
  alertBox.style.background = col;
  alertBox.innerHTML = text;
  if (text == "&nbsp;") {
    alertBox.removeAttribute("style");
  } else {
    alertBox.style.display = "block";
  }
};

const refreshExpiryTimes = async () => {
  const tds = document.getElementsByClassName("tooltip");
  for (let i = 0; i < tds.length; i++) {
    let td = tds[i];
    let span = td.firstChild.firstChild;
    if (span.innerText == "expired") {
      continue;
    }
    let expiryTimeParsed = new Date(td.getAttribute("data-time") * 1000);
    let relativeTime = formatRelativeTime(expiryTimeParsed);
    if (relativeTime == "expired") {
      td.style.color = "light-dark(red, #a01e1e)";
      for (const btn of td.parentElement.lastChild.querySelectorAll("button")) {
        btn.disabled = true;
      }
      td.firstChild.lastChild.remove();
    }
    if (span.innerText != relativeTime) {
      span.innerText = relativeTime;
    }
  }
  if (tds.length > 0) {
    setTimeout(refreshExpiryTimes, 1000);
  }
};

const formatRelativeTime = (timestamp) => {
  const now = new Date();

  const diff = timestamp - now;
  const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });
  if (diff <= 0) {
    return "expired";
  }
  // "Math.abs" accounts for both "past" & "future" scenarios
  for (const u in UNITS) {
    if (Math.abs(diff) > UNITS[u] || u === "second") {
      return rtf.format(Math.round(diff / UNITS[u]), u);
    }
  }
};

const A_SHORT = (s) => `<a href="${SITE_URL}/${s}" target="_blank">${s}</a>`;
const TD = (s, u) => {
  const td = document.createElement("td");
  const div = document.createElement("div");
  div.innerHTML = s;
  td.appendChild(div);
  if (u !== null) td.setAttribute("label", u);
  return td;
};

const TR = (i, row) => {
  const tr = document.createElement("tr");

  const numTD = TD(i, null);
  numTD.setAttribute("name", "numColumn");

  const longlink = row["longlink"];
  const a = document.createElement("a");
  a.href = longlink;
  a.textContent = longlink;
  a.target = "_blank";
  const div = document.createElement("div");
  div.appendChild(a);
  const longTD = document.createElement("td");
  longTD.setAttribute("label", "Long URL");
  longTD.setAttribute("name", "longColumn");
  longTD.appendChild(div);

  const shortlink = row["shortlink"];
  tr.id = shortlink;
  const shortTD = TD(A_SHORT(shortlink), "Short URL");
  shortTD.setAttribute("name", "shortColumn");

  const hitsTD = TD(row["hits"], null);
  hitsTD.setAttribute("label", "Hits");
  hitsTD.setAttribute("name", "hitsColumn");

  const expiryTime = row["expiry_time"];
  let expiryHTML = "-";
  if (expiryTime > 0) {
    const expiryTimeParsed = new Date(expiryTime * 1000);
    const relativeExpiryTime = formatRelativeTime(expiryTimeParsed);
    const accurateExpiryTime = expiryTimeParsed.toLocaleString();
    expiryHTML =
      "<span>" +
      relativeExpiryTime +
      '</span> <span class="tooltiptext">' +
      accurateExpiryTime +
      "</span>";
  }

  let expiryTD = TD(expiryHTML, null);
  if (expiryTime > 0) {
    expiryTD.width = "160px";
    expiryTD.setAttribute("data-time", expiryTime);
    expiryTD.classList.add("tooltip");
  }
  expiryTD.setAttribute("label", "Expiry");
  expiryTD.setAttribute("name", "expiryColumn");

  const actionsTD = document.createElement("td");
  actionsTD.setAttribute("name", "actions");
  actionsTD.setAttribute("label", "Actions");
  const btnGrp = document.createElement("div");
  btnGrp.classList.add("pure-button-group");
  btnGrp.role = "group";
  btnGrp.appendChild(copyButton(shortlink));
  btnGrp.appendChild(qrCodeButton(shortlink));
  btnGrp.appendChild(infoButton(shortlink));
  btnGrp.appendChild(editButton(shortlink, longlink, expiryTime, row["notes"]));
  btnGrp.appendChild(deleteButton(shortlink));
  actionsTD.appendChild(btnGrp);

  for (const td of [numTD, shortTD, longTD, hitsTD, expiryTD, actionsTD]) {
    tr.appendChild(td);
  }
  return tr;
};

const copyShortUrl = (shortLink, doCopy) => {
  const fullLink = `${SITE_URL}/${shortLink}`;
  const linkElt = `<a href=${fullLink} target="_blank">${fullLink}</a>`;

  let copyPromise;
  if (
    typeof ClipboardItem === "function" &&
    typeof navigator.clipboard === "object"
  ) {
    copyPromise = doCopy // We want to use it only for the UI in some cases
      ? navigator.clipboard.writeText(fullLink)
      : Promise.resolve();
  } else {
    copyPromise = Promise.reject(
      new Error("Unable to get access to clipboard."),
    );
  }

  copyPromise
    .then(() =>
      showAlert(
        `Short URL ${linkElt} was copied to clipboard!`,
        "light-dark(green, #1e501e)",
      ),
    )
    .catch((err) => {
      console.log(err);
      showAlert(
        `Could not copy short URL to clipboard, please do it manually: ${linkElt}`,
        "light-dark(red, #a01e1e)",
      );
    });
};

const addHTTPSToLongURL = (id) => {
  const input = document.getElementById(id);
  let url = input.value.trim();
  if (!!url && !hasAllowedScheme(url)) {
    url = "https://" + url;
  }
  input.value = url;
};

const copyButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_COPY_BUTTON;
  btn.title = "Copy Short URL";

  btn.onclick = (e) => {
    e.preventDefault();
    copyShortUrl(shortUrl, true);
  };
  return btn;
};

const editButton = (shortUrl, longUrl, expiry, notes) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_EDIT_BUTTON;
  btn.title = "Edit Short URL";

  btn.onclick = () => {
    document.getElementById("container").style.filter = "blur(2px)";
    const editUrlSpan = document.getElementById("edit-link");
    const editedUrl = document.getElementById("edited-url");
    const editedExpiry = document.getElementById("edited-expiry");
    if (editUrlSpan.textContent != shortUrl) {
      editUrlSpan.textContent = shortUrl;
      document.getElementById("edit-checkbox").checked = false;
      editedUrl.value = longUrl;
      document.getElementById("edited-notes").value = notes;
      if (expiry > 0) {
        const date = new Date(expiry * 1000);
        const local_expiry = new Date(
          date.getTime() - date.getTimezoneOffset() * 60000,
        )
          .toISOString()
          .slice(0, 19);
        editedExpiry.value = local_expiry;
      } else {
        editedExpiry.value = "";
      }
      const min = new Date();
      min.setMinutes(min.getMinutes() + 1);
      const max = new Date();
      max.setFullYear(max.getFullYear() + 5);

      const toLocalISOString = (date) => {
        const pad = (n) => String(n).padStart(2, "0");
        return (
          date.getFullYear() +
          "-" +
          pad(date.getMonth() + 1) +
          "-" +
          pad(date.getDate()) +
          "T" +
          pad(date.getHours()) +
          ":" +
          pad(date.getMinutes())
        );
      };

      editedExpiry.min = toLocalISOString(min);
      editedExpiry.max = toLocalISOString(max);
    }
    document.getElementById("edit-dialog").showModal();
    editedUrl.focus();
  };
  return btn;
};

const infoButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_INFO_BUTTON;
  btn.title = "Show Short URL Info";

  btn.onclick = () => {
    document.getElementById("container").style.filter = "blur(2px)";
    document.getElementById("info-dialog").showModal();
    const row = LOCAL_DATA.filter((row) => row.shortlink == shortUrl)[0];
    document.getElementById("info-short").textContent = row.shortlink;
    document.getElementById("info-long").textContent = row.longlink;
    document.getElementById("info-hits").innerHTML = row.hits;
    const accurateExpiryTime =
      row.expiry_time > 0
        ? new Date(row.expiry_time * 1000).toLocaleString()
        : "Disabled";
    document.getElementById("info-expiry").innerHTML = accurateExpiryTime;
    document.getElementById("info-notes").textContent = row.notes;
  };
  return btn;
};

const qrCodeButton = (shortlink) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_QR_BUTTON;
  btn.title = "Show QR Code";

  const qrWidth = 512;
  const qrPadding = 24;
  const logoWidth = 100;
  const logoRadius = 30;
  const canvasWidth = qrWidth + qrPadding * 2;

  btn.onclick = () => {
    const tmpDiv = document.createElement("div");
    new QRCode(tmpDiv, {
      text: `${SITE_URL}/${shortlink}`,
      correctLevel: QRCode.CorrectLevel.H,
      height: qrWidth,
      width: qrWidth,
    });
    const oldCanvas = tmpDiv.firstChild;

    const newCanvas = document.createElement("canvas");
    newCanvas.height = canvasWidth;
    newCanvas.width = canvasWidth;

    const ctx = newCanvas.getContext("2d");
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, canvasWidth, canvasWidth);
    ctx.drawImage(oldCanvas, qrPadding, qrPadding);

    const img = new Image();
    img.src = "assets/favicon.svg";
    img.onload = () => {
      ctx.fillStyle = "white";
      ctx.beginPath();
      ctx.arc(canvasWidth / 2, canvasWidth / 2, logoRadius * 2, 0, Math.PI * 2);
      ctx.fill();

      ctx.drawImage(
        img,
        (canvasWidth - logoWidth) / 2,
        (canvasWidth - logoWidth) / 2,
        logoWidth,
        logoWidth,
      );

      document.getElementById("qr-code").appendChild(newCanvas);
      document.getElementById("container").style.filter = "blur(2px)";
      document.getElementById("qr-code-dialog").showModal();
      const qrDown = document.getElementById("qr-download-button");
      qrDown.href = newCanvas.toDataURL();
      qrDown.download = `chhoto-qr-${shortlink}.png`;
    };
  };
  return btn;
};

const deleteButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_DELETE_BUTTON;
  btn.title = "Delete Short URL";

  btn.onclick = (e) => {
    e.preventDefault();
    if (confirm("Click OK to delete the entry '" + shortUrl + "'.")) {
      showAlert("&nbsp;", "transparent");
      fetch(prepSubdir(`/api/del/${shortUrl}`), {
        method: "DELETE",
        cache: "no-cache",
      })
        .then(async (res) => {
          if (!res.ok) {
            throw new Error("Could not delete.");
          }
          LOCAL_DATA = LOCAL_DATA.filter(
            (item) => item["shortlink"] != shortUrl,
          );
          if (
            LOCAL_DATA.length <= CUR_PAGE * CONFIG.frontend_page_size &&
            CUR_PAGE > 0
          ) {
            CUR_PAGE -= 1;
          }
          PROCESSING_PAGE_TRANSITION = true;
          displayData();
          managePageControls();
        })
        .catch((err) => {
          console.log("Error:", err);
          showAlert(
            "Unable to delete " + shortUrl + ". Please try again!",
            "light-dark(red, #a01e1e)",
          );
        });
    }
  };
  return btn;
};

const submitForm = () => {
  const form = document.forms.namedItem("new-url-form");
  const longUrl = form.elements["longUrl"];
  const shortUrl = form.elements["shortUrl"];
  const expiryDelay = form.elements["expiryDelay"];
  const data = {
    longlink: longUrl.value,
    shortlink: shortUrl.value,
    expiry_delay: parseInt(expiryDelay.value),
    notes: notes.value,
  };

  const url = prepSubdir("/api/new");

  const payload = {
    method: "POST",
    cache: "no-cache",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(data),
  };

  const cleanPageAfterSubmit = async (ok) => {
    if (ok) {
      longUrl.value = "";
      shortUrl.value = "";
      expiryDelay.value = 0;
      notes.value = "";
      if (ADMIN) {
        const params = new URLSearchParams();
        params.append("page_size", 1);
        const newEntry = await pullData(params);
        if (LOCAL_DATA?.[0]?.shortlink !== newEntry[0].shortlink) {
          LOCAL_DATA.unshift(newEntry[0]);
        }
        CUR_PAGE = 0;
        PROCESSING_PAGE_TRANSITION = true;
        displayData();
        managePageControls();
      }
    }
  };

  let ok = false;
  if (
    typeof ClipboardItem === "function" &&
    typeof navigator.clipboard === "object"
  ) {
    const copyPromise = new ClipboardItem({
      "text/plain": fetch(url, payload)
        .then((res) => {
          ok = res.ok;
          return res.text();
        })
        .then((shortUrl) => {
          if (ok) {
            copyShortUrl(shortUrl, false);
            cleanPageAfterSubmit(ok);
            return new Blob([`${SITE_URL}/${shortUrl}`], {
              type: "text/plain",
            });
          } else {
            showAlert(shortUrl, "light-dark(red, #a01e1e)");
          }
        })
        .catch((err) => {
          console.log("Error:", err);
          if (!alert("Something went wrong! Click OK to refresh page.")) {
            window.location.reload();
          }
        }),
    });
    navigator.clipboard.write([copyPromise]);
  } else {
    // To maintain backwards compatibility, might be removed later
    fetch(url, payload, { cache: "no-cache" })
      .then((res) => {
        ok = res.ok;
        return res.text();
      })
      .then((shortUrl) => {
        if (ok) {
          copyShortUrl(shortUrl, true);
        } else {
          showAlert(shortUrl, "light-dark(red, #a01e1e)");
        }
      })
      .then(() => cleanPageAfterSubmit(ok))
      .catch((err) => {
        console.log("Error:", err);
        if (!alert("Something went wrong! Click OK to refresh page.")) {
          window.location.reload();
        }
      });
  }
};

const submitEdit = () => {
  const urlInput = document.getElementById("edited-url");
  const editUrlSpan = document.getElementById("edit-link");
  const longUrl = urlInput.value;
  const shortUrl = editUrlSpan.textContent;
  const checkBox = document.getElementById("edit-checkbox");
  const notes = document.getElementById("edited-notes").value;
  let expiry = 0;
  const expiry_raw = document.getElementById("edited-expiry").value;
  if (expiry_raw != "") {
    expiry = Math.floor(new Date(expiry_raw).getTime() / 1000);
  }
  if (confirm("Click OK to confirm the edit of '" + shortUrl + "'.")) {
    data = {
      shortlink: shortUrl,
      longlink: longUrl,
      reset_hits: checkBox.checked,
      notes: notes,
      expiry_time: expiry,
    };
    const url = prepSubdir("/api/edit");
    let ok = false;

    fetch(url, {
      method: "PUT",
      cache: "no-cache",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(data),
    })
      .then((res) => {
        ok = res.ok;
        return res.text();
      })
      .then(async (text) => {
        if (!ok) {
          showAlert(JSON.parse(text).reason, "light-dark(red, #a01e1e)");
        } else {
          editUrlSpan.textContent = shortUrl;
          const editedIndex = LOCAL_DATA.findIndex(
            (item) => item["shortlink"] == shortUrl,
          );
          LOCAL_DATA[editedIndex]["longlink"] = longUrl;
          LOCAL_DATA[editedIndex]["notes"] = notes;
          LOCAL_DATA[editedIndex]["expiry_time"] = expiry;
          if (checkBox.checked) {
            LOCAL_DATA[editedIndex]["hits"] = 0;
          }
          checkBox.checked = false;
        }
        document.getElementById("edit-dialog").close();
        displayData();
      })
      .catch((err) => {
        console.log("Error:", err);
        if (!alert("Something went wrong! Click OK to refresh page.")) {
          window.location.reload();
        }
      });
  }
};

const submitLogin = () => {
  const password = document.getElementById("password");
  fetch(prepSubdir("/api/login"), {
    method: "POST",
    cache: "no-cache",
    body: password.value,
  })
    .then(async (res) => {
      switch (res.status) {
        case 200:
          document.getElementById("container").style.filter = "blur(0px)";
          document.getElementById("login-dialog").close();
          password.value = "";
          document.getElementById("wrong-pass").hidden = true;
          ADMIN = true;
          cacheAdmin(true);
          await getConfig();
          await refreshData();
          break;
        case 401:
          document.getElementById("wrong-pass").hidden = false;
          password.focus();
          break;
        default:
          throw new Error("Got status " + res.status);
      }
    })
    .catch((err) => {
      console.log("Error:", err);
      if (!alert("Something went wrong! Click OK to refresh page.")) {
        window.location.reload();
      }
    });
};

const logOut = async () => {
  if (confirm("Click OK to log out.")) {
    await fetch(prepSubdir("/api/logout"), {
      method: "DELETE",
      cache: "no-cache",
    })
      .then(async (res) => {
        if (res.ok) {
          document.getElementById("version-number").hidden = true;
          document.getElementById("admin-button").hidden = true;
          showAlert("&nbsp;", "transparent");
          clearCachedState();
          ADMIN = false;
          LOCAL_DATA = [];
          document.getElementById("table-box").hidden = true;
          document.getElementById("loading-text").hidden = false;
          document.getElementById("url-table").innerHTML = "";
          await refreshData();
        } else {
          showAlert(
            `Logout failed. Please try again!`,
            "light-dark(red, #a01e1e)",
          );
        }
      })
      .catch((err) => {
        console.log("Error:", err);
        if (!alert("Something went wrong! Click OK to refresh page.")) {
          window.location.reload();
        }
      });
  }
};

// This is where loading starts
refreshData()
  .then(() => {
    document.getElementById("longUrl").onblur = () => {
      addHTTPSToLongURL("longUrl");
    };
    document.getElementById("edited-url").onblur = () => {
      addHTTPSToLongURL("edited-url");
    };
    const form = document.forms.namedItem("new-url-form");
    form.onsubmit = (e) => {
      e.preventDefault();
      submitForm();
    };

    document.getElementById("admin-button").onclick = (e) => {
      e.preventDefault();
      if (ADMIN) {
        logOut();
      } else {
        showLogin();
      }
    };

    const editDialog = document.getElementById("edit-dialog");
    editDialog.onclose = () => {
      document.getElementById("container").style.filter = "blur(0px)";
    };
    document.forms.namedItem("edit-form").onsubmit = (e) => {
      e.preventDefault();
      submitEdit();
    };
    document.getElementById("edit-cancel-button").onclick = () => {
      editDialog.close();
    };

    const passEye = document.getElementById("password-eye-button");
    passEye.innerHTML = SVG_OPEN_EYE;
    passEye.onclick = () => {
      const passBox = document.getElementById("password");
      if (passBox.type === "password") {
        passBox.type = "text";
        passEye.innerHTML = SVG_CLOSED_EYE;
      } else {
        passBox.type = "password";
        passEye.innerHTML = SVG_OPEN_EYE;
      }
      document.getElementById("password").focus();
    };

    const prevPageBtn = document.getElementById("prevPageBtn");
    prevPageBtn.innerHTML = SVG_PREV_BUTTON;
    prevPageBtn.onclick = () => {
      gotoPrevPage();
    };
    const nextPageBtn = document.getElementById("nextPageBtn");
    nextPageBtn.innerHTML = SVG_NEXT_BUTTON;
    nextPageBtn.onclick = () => {
      gotoNextPage();
    };

    document.getElementById("filterText").value = "";
    document.forms.namedItem("filter-form").onsubmit = (e) => {
      e.preventDefault();
      refreshWithFilter();
    };

    const infoDialog = document.getElementById("info-dialog");
    document.getElementById("info-close-button").onclick = () => {
      infoDialog.close();
    };
    infoDialog.onclose = () => {
      document.getElementById("container").style.filter = "blur(0px)";
    };

    const qrCodeDialog = document.getElementById("qr-code-dialog");
    document.getElementById("qr-close-button").onclick = () => {
      qrCodeDialog.close();
    };
    qrCodeDialog.onclose = () => {
      document.getElementById("container").style.filter = "blur(0px)";
      document.getElementById("qr-code").innerHTML = "";
    };

    document.forms.namedItem("login-form").onsubmit = (e) => {
      e.preventDefault();
      submitLogin();
    };
  })
  .catch((err) => {
    console.log("Something went wrong:", err);
    if (!alert("Something went wrong! Click OK to refresh page.")) {
      window.location.reload();
    }
  });
