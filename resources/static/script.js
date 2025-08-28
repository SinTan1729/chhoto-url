// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

let VERSION = null;
let SITE_URL = "-";
let CONFIG = null;
let SUBDIR = null;
let ADMIN = false;

// Buttons
SVG_COPY_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M9 3.25A5.75 5.75 0 0 0 3.25 9v7.107a.75.75 0 0 0 1.5 0V9A4.25 4.25 0 0 1 9 4.75h7.013a.75.75 0 0 0 0-1.5z"/><path fill="currentColor" fill-rule="evenodd" d="M18.403 6.793a44.372 44.372 0 0 0-9.806 0a2.011 2.011 0 0 0-1.774 1.76a42.581 42.581 0 0 0 0 9.894a2.01 2.01 0 0 0 1.774 1.76c3.241.362 6.565.362 9.806 0a2.01 2.01 0 0 0 1.774-1.76a42.579 42.579 0 0 0 0-9.894a2.011 2.011 0 0 0-1.774-1.76M8.764 8.284c3.13-.35 6.342-.35 9.472 0a.51.51 0 0 1 .45.444a40.95 40.95 0 0 1 0 9.544a.51.51 0 0 1-.45.444c-3.13.35-6.342.35-9.472 0a.511.511 0 0 1-.45-.444a40.95 40.95 0 0 1 0-9.544a.511.511 0 0 1 .45-.444" clip-rule="evenodd"/></svg>`;
SVG_EDIT_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" fill-rule="evenodd" d="M21.455 5.416a.75.75 0 0 1-.096.943l-9.193 9.192a.75.75 0 0 1-.34.195l-3.829 1a.75.75 0 0 1-.915-.915l1-3.828a.778.778 0 0 1 .161-.312L17.47 2.47a.75.75 0 0 1 1.06 0l2.829 2.828a.756.756 0 0 1 .096.118m-1.687.412L18 4.061l-8.518 8.518l-.625 2.393l2.393-.625z" clip-rule="evenodd"/><path fill="currentColor" d="M19.641 17.16a44.4 44.4 0 0 0 .261-7.04a.403.403 0 0 1 .117-.3l.984-.984a.198.198 0 0 1 .338.127a45.91 45.91 0 0 1-.21 8.372c-.236 2.022-1.86 3.607-3.873 3.832a47.77 47.77 0 0 1-10.516 0c-2.012-.225-3.637-1.81-3.873-3.832a45.922 45.922 0 0 1 0-10.67c.236-2.022 1.86-3.607 3.873-3.832a47.75 47.75 0 0 1 7.989-.213a.2.2 0 0 1 .128.34l-.993.992a.402.402 0 0 1-.297.117a46.164 46.164 0 0 0-6.66.255a2.89 2.89 0 0 0-2.55 2.516a44.421 44.421 0 0 0 0 10.32a2.89 2.89 0 0 0 2.55 2.516c3.355.375 6.827.375 10.183 0a2.89 2.89 0 0 0 2.55-2.516"/></svg>`;
SVG_DELETE_BUTTON = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M10 2.25a.75.75 0 0 0-.75.75v.75H5a.75.75 0 0 0 0 1.5h14a.75.75 0 0 0 0-1.5h-4.25V3a.75.75 0 0 0-.75-.75zM13.06 15l1.47 1.47a.75.75 0 1 1-1.06 1.06L12 16.06l-1.47 1.47a.75.75 0 1 1-1.06-1.06L10.94 15l-1.47-1.47a.75.75 0 1 1 1.06-1.06L12 13.94l1.47-1.47a.75.75 0 1 1 1.06 1.06z"/><path fill="currentColor" fill-rule="evenodd" d="M5.991 7.917a.75.75 0 0 1 .746-.667h10.526a.75.75 0 0 1 .746.667l.2 1.802c.363 3.265.363 6.56 0 9.826l-.02.177a2.853 2.853 0 0 1-2.44 2.51a27.04 27.04 0 0 1-7.498 0a2.853 2.853 0 0 1-2.44-2.51l-.02-.177a44.489 44.489 0 0 1 0-9.826zm1.417.833l-.126 1.134a42.99 42.99 0 0 0 0 9.495l.02.177a1.353 1.353 0 0 0 1.157 1.191c2.35.329 4.733.329 7.082 0a1.353 1.353 0 0 0 1.157-1.19l.02-.178c.35-3.155.35-6.34 0-9.495l-.126-1.134z" clip-rule="evenodd"/></svg>`;

// in miliseconds
const UNITS = {
  year: 31536000000,
  month: 2592000000,
  day: 86400000,
  hour: 3600000,
  minute: 60000,
  second: 1000,
};

const prepSubdir = (link) => {
  if (!SUBDIR) {
    const thisPage = new URL(window.location.href);
    SUBDIR = thisPage.pathname.replace(/\/admin\/manage\/$/, "/");
  }
  return (SUBDIR + link).replace("//", "/");
};

const hasProtocol = (url) => {
  const regex = /[A-Za-z][A-Za-z0-9\+\-\.]*\:(?:\/\/)?.*\D.*/; // RFC 2396 Appendix A
  return regex.test(url);
};

const getConfig = async () => {
  if (!CONFIG) {
    CONFIG = await fetch(prepSubdir("/api/getconfig"), { cache: "no-cache" })
      .then((res) => res.json())
      .catch((err) => {
        console.log("Error while fetching config.");
      });
    if (CONFIG.site_url == null) {
      SITE_URL = window.location.host;
    } else {
      SITE_URL = CONFIG.site_url
        .replace(/\/$/, "")
        .replace(/^"/, "")
        .replace(/"$/, "");
    }

    if (!hasProtocol(SITE_URL)) {
      SITE_URL = window.location.protocol + "//" + SITE_URL;
    }

    VERSION = CONFIG.version;
  }
};

const showVersion = () => {
  const link = document.getElementById("version-number");
  link.innerText = "v" + VERSION;
  link.href =
    "https://github.com/SinTan1729/chhoto-url/releases/tag/" + VERSION;
  link.hidden = false;
};

const showLogin = () => {
  document.getElementById("container").style.filter = "blur(2px)";
  document.getElementById("login-dialog").showModal();
  document.getElementById("password").focus();
};

const refreshData = async () => {
  try {
    const res = await fetch(prepSubdir("/api/all"), { cache: "no-cache" });
    switch (res.status) {
      case 200:
        const data = await res.json();
        await getConfig();
        ADMIN = true;
        displayData(data.reverse());
        break;
      case 401:
        const loading_text = document.getElementById("loading-text");
        const admin_button = document.getElementById("admin-button");
        document.getElementById("table-box").hidden = true;
        loading_text.hidden = false;
        admin_button.innerText = "login";

        const errorMsg = await res.text();
        document.getElementById("url-table").innerHTML = "";
        if (errorMsg.startsWith("Using public mode.")) {
          admin_button.hidden = false;
          loading_text.innerHTML = "Using public mode.";
          const expiry = parseInt(errorMsg.split(" ").pop());
          if (expiry > 0) {
            loading_text.innerHTML +=
              " Unless chosen a shorter expiry time, submitted links will automatically expire ";
            const time = new Date();
            time.setSeconds(time.getSeconds() + expiry);
            loading_text.innerHTML += formatRelativeTime(time) + ".";
          }
          await getConfig();
          showVersion();
          updateInputBox();
        } else {
          showLogin();
        }
        break;
      default:
        if (!alert("Something went wrong! Click Ok to refresh page.")) {
          window.location.reload();
        }
    }
  } catch (err) {
    console.log(err);
    showAlert(
      `Could not copy short URL to clipboard, please do it manually: ${link_elt}`,
      "light-dark(red, #ff1a1a)",
    );
  }
};

const updateInputBox = () => {
  if (CONFIG.allow_capital_letters) {
    const input_box = document.getElementById("shortUrl");
    input_box.pattern = "[A-Za-z0-9\-_]+";
    input_box.title = "Only A-Z, a-z, 0-9, - and _ are allowed";
    input_box.placeholder = "Only A-Z, a-z, 0-9, - and _ are allowed";
  }
};

const displayData = (data) => {
  showVersion();
  const admin_button = document.getElementById("admin-button");
  admin_button.innerText = "logout";
  admin_button.hidden = false;
  updateInputBox();

  const table_box = document.getElementById("table-box");
  const loading_text = document.getElementById("loading-text");
  const table = document.getElementById("url-table");

  if (data.length === 0) {
    table_box.hidden = true;
    loading_text.innerHTML = "No active links.";
    loading_text.hidden = false;
  } else {
    loading_text.hidden = true;
    table_box.hidden = false;
    table.innerHTML = "";
    for (const [i, row] of data.entries()) {
      table.appendChild(TR(i + 1, row));
    }
    setTimeout(refreshExpiryTimes, 1000);
  }
};

const showAlert = (text, col) => {
  document.getElementById("alert-box")?.remove();
  const controls = document.getElementById("controls");
  const alertBox = document.createElement("p");
  alertBox.id = "alert-box";
  alertBox.style.color = col;
  alertBox.innerHTML = text;
  alertBox.style.display = "block";
  controls.appendChild(alertBox);
};

const refreshExpiryTimes = async () => {
  const tds = document.getElementsByClassName("tooltip");
  for (let i = 0; i < tds.length; i++) {
    let td = tds[i];
    let expiryTimeParsed = new Date(td.getAttribute("data-time") * 1000);
    let relativeTime = formatRelativeTime(expiryTimeParsed);
    if (relativeTime == "expired") {
      td.style.color = "light-dark(red, #ff1a1a)";
    }
    let div = td.firstChild;
    div.innerHTML = div.innerHTML.replace(div.innerText, relativeTime);
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
  const longTD = TD(A_LONG(longlink), "Long URL");

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
    expiryTimeParsed = new Date(expiryTime * 1000);
    const relativeExpiryTime = formatRelativeTime(expiryTimeParsed);
    const accurateExpiryTime = expiryTimeParsed.toLocaleString();
    expiryHTML =
      relativeExpiryTime +
      '<span class="tooltiptext">' +
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
  btnGrp.appendChild(editButton(shortlink, longlink));
  btnGrp.appendChild(deleteButton(shortlink));
  actionsTD.appendChild(btnGrp);

  for (const td of [numTD, shortTD, longTD, hitsTD, expiryTD, actionsTD]) {
    tr.appendChild(td);
  }
  return tr;
};

const copyShortUrl = async (short_link) => {
  const full_link = `${SITE_URL}/${short_link}`;
  const link_elt = `<a href=${full_link}>${full_link}</a>`;
  try {
    await navigator.clipboard.writeText(full_link);
    showAlert(
      `Short URL ${link_elt} was copied to clipboard!`,
      "light-dark(green, #72ff72)",
    );
  } catch (err) {
    console.log(err);
    showAlert(
      `Could not copy short URL to clipboard, please do it manually: ${link_elt}`,
      "light-dark(red, #ff1a1a)",
    );
  }
};

const addHTTPSToLongURL = () => {
  const input = document.getElementById("longUrl");
  let url = input.value.trim();
  if (!!url && !hasProtocol(url)) {
    url = "https://" + url;
  }
  input.value = url;
  return input;
};

const A_LONG = (s) => `<a href='${s}'>${s}</a>`;
const A_SHORT = (s) => `<a href="${SITE_URL}/${s}">${s}</a>`;

const copyButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = `${SVG_COPY_BUTTON}`;

  btn.onclick = (e) => {
    e.preventDefault();
    copyShortUrl(shortUrl);
  };
  return btn;
};

const editButton = (shortUrl, longUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = `${SVG_EDIT_BUTTON}`;

  btn.onclick = () => {
    document.getElementById("container").style.filter = "blur(2px)";
    document.getElementById("edit-dialog").showModal();
    const editUrlSpan = document.getElementById("edit-link");
    const editedUrl = document.getElementById("edited-url");
    if (editUrlSpan.textContent != shortUrl) {
      editUrlSpan.textContent = shortUrl;
      document.getElementById("edit-checkbox").checked = false;
      editedUrl.value = longUrl;
    }
    editedUrl.focus();
  };
  return btn;
};

const deleteButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = `${SVG_DELETE_BUTTON}`;

  btn.onclick = (e) => {
    e.preventDefault();
    if (confirm("Do you want to delete the entry " + shortUrl + "?")) {
      showAlert("&nbsp;", "black");
      fetch(prepSubdir(`/api/del/${shortUrl}`), {
        method: "DELETE",
        cache: "no-cache",
      })
        .then(async (res) => {
          if (!res.ok) {
            throw new Error("Could not delete.");
          }
          await refreshData();
        })
        .catch((err) => {
          console.log("Error:", err);
          showAlert(
            "Unable to delete " + shortUrl + ". Please try again!",
            "light-dark(red, #ff1a1a)",
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
  };

  const url = prepSubdir("/api/new");
  let ok = false;

  fetch(url, {
    method: "POST",
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
        showAlert(text, "light-dark(red, #ff1a1a)");
      } else {
        await copyShortUrl(text);
        longUrl.value = "";
        shortUrl.value = "";
        expiryDelay.value = 0;
      }
      await refreshData();
    })
    .catch((err) => {
      console.log("Error:", err);
      if (!alert("Something went wrong! Click Ok to refresh page.")) {
        window.location.reload();
      }
    });
};

const submitEdit = () => {
  const urlInput = document.getElementById("edited-url");
  const editUrlSpan = document.getElementById("edit-link");
  const longUrl = urlInput.value;
  const shortUrl = editUrlSpan.textContent;
  const checkBox = document.getElementById("edit-checkbox");
  if (confirm("Are you sure that you want to edit " + shortUrl + "?")) {
    data = {
      shortlink: shortUrl,
      longlink: longUrl,
      reset_hits: checkBox.checked,
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
          showAlert(text, "light-dark(red, #ff1a1a)");
        } else {
          document.getElementById("edit-dialog").close();
          editUrlSpan.textContent = shortUrl;
          checkBox.checked = false;
        }
        await refreshData();
      })
      .catch((err) => {
        console.log("Error:", err);
        if (!alert("Something went wrong! Click Ok to refresh page.")) {
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
      if (!alert("Something went wrong! Click Ok to refresh page.")) {
        window.location.reload();
      }
    });
};

const logOut = async () => {
  if (confirm("Are you sure you want to log out?")) {
    await fetch(prepSubdir("/api/logout"), {
      method: "DELETE",
      cache: "no-cache",
    })
      .then(async (res) => {
        if (res.ok) {
          document.getElementById("version-number").hidden = true;
          document.getElementById("admin-button").hidden = true;
          showAlert("&nbsp;", "black");
          ADMIN = false;
          await refreshData();
        } else {
          showAlert(
            `Logout failed. Please try again!`,
            "light-dark(red, #ff1a1a)",
          );
        }
      })
      .catch((err) => {
        console.log("Error:", err);
        if (!alert("Something went wrong! Click Ok to refresh page.")) {
          window.location.reload();
        }
      });
  }
};

// This is where loading starts
refreshData()
  .then(() => {
    document.getElementById("longUrl").onblur = addHTTPSToLongURL;
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
    document.forms.namedItem("login-form").onsubmit = (e) => {
      e.preventDefault();
      submitLogin();
    };
  })
  .catch((err) => {
    console.log("Something went wrong:", err);
    if (!alert("Something went wrong! Click Ok to refresh page.")) {
      window.location.reload();
    }
  });
