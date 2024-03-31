const prepSubdir = (link) => {
    let thisPage = new URL(window.location.href);
    let subdir = thisPage.pathname;
    let out = (subdir+link).replace('//','/');
    console.log(out);
    return (subdir+link).replace('//','/');
}

const getSiteUrl = async () => await fetch(prepSubdir("/api/siteurl"))
    .then(res => res.text())
    .then(text => {
        if (text == "unset") {
            return window.location.host.replace(/\/$/,'');
        }
        else {
            return text.replace(/\/$/,'').replace(/^"/,'').replace(/"$/,'');
        }
    });

const getVersion = async () => await fetch(prepSubdir("/api/version"))
    .then(res => res.text())
    .then(text => {
        return text;
    });

const refreshData = async () => {
    let reply = await fetch(prepSubdir("/api/all")).then(res => res.text());
    if (reply == "logged_out") {
        console.log("logged_out");
        document.getElementById("container").style.filter = "blur(2px)"
        document.getElementById("login-dialog").showModal();
        document.getElementById("password").focus();
    } else {
        data = reply
            .split("\n")
            .filter(line => line !== "")
            .map(line => line.split(","))
            .map(arr => ({
                short: arr[0],
                long: arr[1],
                hits: arr[2]
            }));

        displayData(data);
    }
};

const displayData = async (data) => {
    let version = await getVersion();
    link = document.getElementById("version-number")
    link.innerText = "v" + version;
    link.href = "https://github.com/SinTan1729/chhoto-url/releases/tag/" + version;
    link.hidden = false;

    let site = await getSiteUrl();

    table_box = document.querySelector(".pure-table");
    loading_text = document.getElementsByName("loading-text")[0];

    if (data.length == 0) {
        table_box.style.visibility = "hidden";
        loading_text.style.display = "block";
        loading_text.innerHTML = "No active links.";
    }
    else {
        loading_text.style.display = "none";
        const table = document.querySelector("#url-table");
        if (!window.isSecureContext) {
            const shortUrlHeader = document.getElementById("short-url-header");
            shortUrlHeader.innerHTML = "Short URL<br>(right click and copy)";
        }
        table_box.style.visibility = "visible";
        table.innerHTML = ''; // Clear
        data.forEach(tr => table.appendChild(TR(tr, site)));
    }
};

const showAlert = async (text, col) => {
    document.getElementById("alert-box")?.remove();
    const controls = document.querySelector(".pure-controls");
    const alertBox = document.createElement("p");
    alertBox.id = "alert-box";
    alertBox.style.color = col;
    alertBox.innerHTML = text;
    controls.appendChild(alertBox);
};

const TR = (row, site) => {
    const tr = document.createElement("tr");
    const longTD = TD(A_LONG(row.long), "Long URL");
    var shortTD = null;
    if (window.isSecureContext) {
        shortTD = TD(A_SHORT(row.short, site), "Short URL");
    }
    else {
        shortTD = TD(A_SHORT_INSECURE(row.short, site), "Short URL");
    }
    hitsTD = TD(row.hits);
    hitsTD.setAttribute("label", "Hits");
    hitsTD.setAttribute("name", "hitsColumn");
    const btn = deleteButton(row.short);

    tr.appendChild(shortTD);
    tr.appendChild(longTD);
    tr.appendChild(hitsTD);
    tr.appendChild(btn);

    return tr;
};

const copyShortUrl = async (link) => {
    const site = await getSiteUrl();
    try {
        navigator.clipboard.writeText(`${site}/${link}`);
        showAlert(`Short URL ${link} was copied to clipboard!`, "green");
    } catch (e) {
        console.log(e);
        showAlert("Could not copy short URL to clipboard, please do it manually.", "red");
    }

};

const addProtocol = (input) => {
    var url = input.value.trim();
    if (url != "" && !~url.indexOf("://") && !~url.indexOf("magnet:")) {
        url = "https://" + url;
    }
    input.value = url;
    return input
}

const A_LONG = (s) => `<a href='${s}'>${s}</a>`;
const A_SHORT = (s, t) => `<a href="javascript:copyShortUrl('${s}');">${s}</a>`;
const A_SHORT_INSECURE = (s, t) => `<a href="${t}/${s}">${s}</a>`;

const deleteButton = (shortUrl) => {
    const td = document.createElement("td");
    const div = document.createElement("div");
    const btn = document.createElement("button");

    btn.innerHTML = "&times;";

    btn.onclick = e => {
        e.preventDefault();
        if (confirm("Do you want to delete the entry " + shortUrl + "?")) {
            document.getElementById("alert-box")?.remove();
            showAlert("&nbsp;", "black");
            fetch(prepSubdir(`/api/del/${shortUrl}`), {
                method: "DELETE"
            }).then(_ => refreshData());
        }
    };
    td.setAttribute("name", "deleteBtn");
    td.setAttribute("label", "Delete");
    div.appendChild(btn);
    td.appendChild(div);
    return td;
};

const TD = (s, u) => {
    const td = document.createElement("td");
    const div = document.createElement("div");
    div.innerHTML = s;
    td.appendChild(div);
    td.setAttribute("label", u);
    return td;
};

const submitForm = () => {
    const form = document.forms.namedItem("new-url-form");
    const longUrl = form.elements["longUrl"];
    const shortUrl = form.elements["shortUrl"];

    const url = prepSubdir("/api/new");

    fetch(url, {
        method: "POST",
        body: `${longUrl.value};${shortUrl.value}`
    })
        .then(res => {
            if (!res.ok) {
                showAlert("Short URL is not valid or it's already in use!", "red");
                return "error";
            }
            else {
                return res.text();
            }
        }).then(text => {
            if (text != "error") {
                copyShortUrl(text);
                longUrl.value = "";
                shortUrl.value = "";
                refreshData();
            }
        });
};

const submitLogin = () => {
    const password = document.getElementById("password");
    fetch(prepSubdir("/api/login"), {
        method: "POST",
        body: password.value
    }).then(res => {
        if (res.ok) {
            document.getElementById("container").style.filter = "blur(0px)"
            document.getElementById("login-dialog").remove();
            refreshData();
        } else {
            const wrongPassBox = document.getElementById("wrong-pass");
            wrongPassBox.innerHTML = "Wrong password!";
            wrongPassBox.style.color = "red";
            password.focus();
        }
    })
}

(async () => {
    await refreshData();

    const form = document.forms.namedItem("new-url-form");
    form.onsubmit = e => {
        e.preventDefault();
        submitForm();
    }

    const login_form = document.forms.namedItem("login-form");
    login_form.onsubmit = e => {
        e.preventDefault();
        submitLogin();
    }
})();
