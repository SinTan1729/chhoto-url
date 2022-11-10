const refreshData = async () => {
    let data = await fetch("/api/all").then(res => res.text());
    data = data
        .split("\n")
        .filter(line => line !== "")
        .map(line => line.split(","))
        .map(arr => ({
            short: arr[0],
            long: arr[1],
            hits: arr[2]
        }));

    displayData(data);
};

const displayData = (data) => {
    table_box = document.querySelector(".pure-table");
    if (data.length == 0) {
        table_box.style.visibility = "hidden";
    }
    else {
        const table = document.querySelector("#url-table");
        table_box.style.visibility = "visible";
        table.innerHTML = ''; // Clear
        data.map(TR)
            .forEach(tr => table.appendChild(tr));
    }
};

const addErrBox = async () => {
    const controls = document.querySelector(".pure-controls");
    const errBox = document.createElement("p");
    errBox.setAttribute("id", "errBox");
    errBox.setAttribute("style", "color:red");
    errBox.innerHTML = "Short URL not valid or already in use!";
    controls.appendChild(errBox);
}

const TR = (row) => {
    const tr = document.createElement("tr");
    const longTD = TD(A(row.long));
    const shortTD = TD(A_INT(row.short));
    const hitsTD = TD(row.hits);
    const btn = deleteButton(row.short);

    tr.appendChild(shortTD);
    tr.appendChild(longTD);
    tr.appendChild(hitsTD);
    tr.appendChild(btn);

    return tr;
};

const A = (s) => `<a href='${s}'>${s}</a>`;
const A_INT = (s) => `<a href='/${s}'>${window.location.host}/${s}</a>`;

const deleteButton = (shortUrl) => {
    const td = document.createElement("td");
    const btn = document.createElement("button");

    btn.innerHTML = "&times;";

    btn.onclick = e => {
        e.preventDefault();
        if (confirm("Do you want to delete the entry " + shortUrl + "?")) {
            fetch(`/api/${shortUrl}`, {
                method: "DELETE"
            }).then(_ => refreshData());
        }
    };
    td.setAttribute("name", "deleteBtn");
    td.appendChild(btn);
    return td;
};

const TD = (s) => {
    const td = document.createElement("td");
    const div = document.createElement("div");
    div.innerHTML = s;
    td.appendChild(div);
    return td;
};

const submitForm = () => {
    const form = document.forms.namedItem("new-url-form");
    const longUrl = form.elements["longUrl"];
    const shortUrl = form.elements["shortUrl"];

    const url = `/api/new`;

    fetch(url, {
        method: "POST",
        body: `${longUrl.value};${shortUrl.value}`
    })
        .then((res) => {
            if (!res.ok) {
                if (document.getElementById("errBox") == null) {
                    addErrBox();
                }
            }
            else {
                document.getElementById("errBox")?.remove();
                longUrl.value = "";
                shortUrl.value = "";

                refreshData();
            }
        });

};

(async () => {
    await refreshData();
    const form = document.forms.namedItem("new-url-form");
    form.onsubmit = e => {
        e.preventDefault();
        submitForm();
    }
})();
