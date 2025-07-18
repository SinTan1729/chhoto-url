// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

let VERSION = null;
let SITE_URL = "-";
let CONFIG = null;
let SUBDIR = null;
let ADMIN = false;

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

const getConfig = async () => {
  if (!CONFIG) {
    CONFIG = await fetch(prepSubdir("/api/getconfig"))
      .then((res) => res.json())
      .catch((err) => {
        console.log("Error while fetching config.");
      });
    if (CONFIG.site_url == null) {
      SITE_URL = window.location.host.replace(/\/$/, "");
    } else {
      SITE_URL = CONFIG.site_url
        .replace(/\/$/, "")
        .replace(/^"/, "")
        .replace(/"$/, "");
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
    const res = await fetch(prepSubdir("/api/all"));
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
    if (!window.isSecureContext) {
      const shortUrlHeader = document.getElementById("short-url-header");
      shortUrlHeader.innerHTML = "Short URL<br>(right click and copy)";
    }
    table_box.hidden = false;
    table.innerHTML = "";
    data.forEach((tr) => table.appendChild(TR(tr)));
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

const TD = (s, u, t) => {
  const td = document.createElement("td");
  const div = document.createElement("div");
  div.innerHTML = s;
  if (t != null) {
    div.onclick = async (e) => {
      e.preventDefault();
      await copyShortUrl(t);
    };
  }
  td.appendChild(div);
  if (u !== null) td.setAttribute("label", u);
  return td;
};

const TR = (row) => {
  const tr = document.createElement("tr");
  const longTD = TD(A_LONG(row["longlink"]), "Long URL", null);
  let shortTD;
  const isSafari =
    /Safari/.test(navigator.userAgent) &&
    /Apple Computer/.test(navigator.vendor);
  // For now, we disable copying on WebKit due to a possible bug. Manual copying is enabled instead.
  // Take a look at https://github.com/SinTan1729/chhoto-url/issues/36
  if (window.isSecureContext && !isSafari) {
    let shortlink = row["shortlink"];
    shortTD = TD(A_SHORT(shortlink, "Short URL"), "Short URL", shortlink);
  } else {
    shortTD = TD(A_SHORT_INSECURE(row["shortlink"]), "Short URL", null);
  }
  const hitsTD = TD(row["hits"], null, null);
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

  let expiryTD = TD(expiryHTML, null, null);
  if (expiryTime > 0) {
    expiryTD.width = "160px";
    expiryTD.setAttribute("data-time", expiryTime);
    expiryTD.classList.add("tooltip");
  }
  expiryTD.setAttribute("label", "Expiry");
  expiryTD.setAttribute("name", "expiryColumn");

  const btn = deleteButton(row["shortlink"]);

  tr.appendChild(shortTD);
  tr.appendChild(longTD);
  tr.appendChild(hitsTD);
  tr.appendChild(expiryTD);
  tr.appendChild(btn);

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

const addProtocol = () => {
  const input = document.getElementById("longUrl");
  let url = input.value.trim();
  if (url !== "" && !~url.indexOf("://") && !~url.indexOf("magnet:")) {
    url = "https://" + url;
  }
  input.value = url;
  return input;
};

const A_LONG = (s) => `<a href='${s}'>${s}</a>`;
const A_SHORT = (s) => `<a href="#!">${s}</a>`;
const A_SHORT_INSECURE = (s, t) => `<a href="${t}/${s}">${s}</a>`;

const deleteButton = (shortUrl) => {
  const td = document.createElement("td");
  const div = document.createElement("div");
  const btn = document.createElement("button");

  btn.innerHTML = "&times;";

  btn.onclick = (e) => {
    e.preventDefault();
    if (confirm("Do you want to delete the entry " + shortUrl + "?")) {
      showAlert("&nbsp;", "black");
      fetch(prepSubdir(`/api/del/${shortUrl}`), {
        method: "DELETE",
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
  td.setAttribute("name", "deleteBtn");
  td.setAttribute("label", "Delete");
  div.appendChild(btn);
  td.appendChild(div);
  return td;
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
        await refreshData();
      } else {
        await copyShortUrl(text);
        longUrl.value = "";
        shortUrl.value = "";
        expiryDelay.value = 0;
        await refreshData();
      }
    })
    .catch((err) => {
      console.log("Error:", err);
      if (!alert("Something went wrong! Click Ok to refresh page.")) {
        window.location.reload();
      }
    });
};

const submitLogin = () => {
  const password = document.getElementById("password");
  fetch(prepSubdir("/api/login"), {
    method: "POST",
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
  await fetch(prepSubdir("/api/logout"), { method: "DELETE" })
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
};

// This is where loading starts
refreshData()
  .then(() => {
    document.getElementById("longUrl").onblur = addProtocol;
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

    const login_form = document.forms.namedItem("login-form");
    login_form.onsubmit = (e) => {
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
