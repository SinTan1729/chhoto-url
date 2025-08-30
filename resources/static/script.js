// SPDX-FileCopyrightText: 2023 Sayantan Santra <sayantan.santra689@gmail.com>
// SPDX-License-Identifier: MIT

let VERSION = null;
let SITE_URL = "-";
let CONFIG = null;
let SUBDIR = null;
let ADMIN = false;

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
      for (const btn of td.parentElement.lastChild.querySelectorAll("button")) {
        btn.disabled = true;
      }
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
  btnGrp.appendChild(qrCodeButton(shortlink));
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
  btn.innerHTML = SVG_COPY_BUTTON;

  btn.onclick = (e) => {
    e.preventDefault();
    copyShortUrl(shortUrl);
  };
  return btn;
};

const editButton = (shortUrl, longUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_EDIT_BUTTON;

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

const qrCodeButton = (shortlink) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_QR_BUTTON;

  btn.onclick = () => {
    const tmpDiv = document.createElement("div");
    new QRCode(tmpDiv, {
      text: `${SITE_URL}/${shortlink}`,
      correctLevel: QRCode.CorrectLevel.H,
    });
    const oldCanvas = tmpDiv.firstChild;

    const padding = "12";
    const newCanvas = document.createElement("canvas");
    newCanvas.height = 280;
    newCanvas.width = 280;

    const ctx = newCanvas.getContext("2d");
    ctx.fillStyle = "white";
    ctx.fillRect(0, 0, 280, 280);
    ctx.drawImage(oldCanvas, 12, 12);

    const img = new Image();
    img.src = prepSubdir("/assets/favicon.svg");
    img.onload = () => {
      ctx.fillStyle = "white";
      ctx.beginPath();
      ctx.arc(140, 140, 30, 0, Math.PI * 2);
      ctx.fill();

      const imgWidth = 50;
      const imgHeight = 50;
      ctx.drawImage(img, 115, 115, 50, 50);

      document.getElementById("qr-code").appendChild(newCanvas);
      const qrDown = document.getElementById("qr-download");
      qrDown.href = newCanvas.toDataURL();
      qrDown.download = `chhoto-qr-${shortlink}.png`;
      document.getElementById("container").style.filter = "blur(2px)";
      document.getElementById("qr-code-dialog").showModal();
    };
  };
  return btn;
};

const deleteButton = (shortUrl) => {
  const btn = document.createElement("button");
  btn.classList.add("svg-button");
  btn.innerHTML = SVG_DELETE_BUTTON;

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
    };

    const qrCodeDialog = document.getElementById("qr-code-dialog");
    document.getElementById("qr-close").onclick = () => {
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
    if (!alert("Something went wrong! Click Ok to refresh page.")) {
      window.location.reload();
    }
  });
