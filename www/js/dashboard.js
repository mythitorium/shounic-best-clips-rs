//
//
//
//
//


var t;
var configObj;
var loadedConfigForTheFirstTime = false;
var loadedDashboardForTheFirstTime = false;
const inputIds = ["allow_voting", "unix_deadline", "voting_stage", "limit_votes", "videos_per_vote", "elimination_threshold", "include_usernames"];
var liveNumberOfChangedInputs = 0;


//
//
//
//
//

async function request(endpoint, queries, payload, method) {
    let url;
    let options = {};
    options.method = method;
    if (!(payload === null)) { 
        options.body = JSON.stringify(payload); 
        options.headers = {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        };
    }
    console.log(queries);
    console.log(new URLSearchParams(queries).toString());
    if ((queries === null)) { url = endpoint } else { url = endpoint + "?" + (new URLSearchParams(queries)).toString(); }
    console.log(url);

    return await window.fetch(url, options).then((response) => {
        return [response.status, response.statusText, response.json()];
    }).then((responseArr) => {
        let json = responseArr[2];
        let status = responseArr[0];
        let fallbackText = responseArr[1];

        if (status === 200) {
            clearErrField();
            if (json === null) {
                return {};
            } else {
                return json;
            }
        } else {
            if (Object.hasOwn(json, "message")) {
                getById("errFieldPLogin").innerHTML = status + ": " + json.message;
                getById("errFieldP").innerHTML = status + ": " + json.message;
            } else {
                getById("errFieldPLogin").innerHTML = status + ": " + fallbackText;
                getById("errFieldP").innerHTML = status + ": " + fallbackText;
            }
            return null;
        }
    });
}


function getById(id) {
    return document.getElementById(id);
}


function clearErrField() {
    getById("errFieldP").innerHTML = "";
}


function clearLoginFields() {
    getById("loginPassword").value = "";
    getById("loginUsername").value = "";
}


function swapBetween(toId, fromId, classString) {
    getById(toId).classList.add(classString);
    getById(fromId).classList.remove(classString);
}

//
//
//
//
//

(async () => {
    clearInputs();
    clearLoginFields();

    getById("configRegion").classList.add("unloaded");
    getById("dashboardRegion").classList.add("unloaded");

    swapBetween("dashboard", "login", "hidden");
} )();


//
//
//
//
//


function tabClicked(tab) {
    clearErrField();
    switch (tab) {
        case "config":
            swapBetween("configTab", "dashboardTab", "selected");
            swapBetween("dashboardRegion", "configRegion", "hidden");
            if (loadedConfigForTheFirstTime === false) {
                getConfigParameters();
            }
            break;
        case "dash":
            swapBetween("dashboardTab", "configTab", "selected");
            swapBetween("configRegion", "dashboardRegion", "hidden");
            break;
        default:
            break;
    }
}


async function getConfigParameters() {
    let payload = await request("/server/config", { token: t }, null, "GET");
    if (!(payload === null)) {
        configObj = payload;
        if (loadedConfigForTheFirstTime === false) {
            loadedConfigForTheFirstTime = true;
            revertConfigChanges();
            getById("configRegion").classList.remove("unloaded");
        }
    }
}


async function login() {
    let payload = await request("/admin/login", null, { username: getById("loginUsername").value, password: getById("loginPassword").value }, "POST");
    if (!(payload === null)) {
        t = payload.token;
        completeLogin();
    } else {
        clearLoginFields();
    }
}

function clearInputs() {
    for (id of inputIds) {
        getById(id).value = '';
    }
}


function completeLogin() {
    swapBetween("login", "dashboard", "hidden");
    tabClicked("config");
}



function inputChanged(inputId) {
    console.log('--- ' + inputId);
    console.log(configObj[inputId]);

    const input = getById(inputId);
    const marker = getById(inputId + "Marker");
    if (isInputDifferent(inputId)) {
        marker.innerHTML = " *";
        liveNumberOfChangedInputs += 1;
    } else {
        marker.innerHTML = "";
        liveNumberOfChangedInputs -= 1;
    }

    getById("postConfigButton").disabled = (liveNumberOfChangedInputs === 0);
    getById("discardConfigChangesButton").disabled = (liveNumberOfChangedInputs === 0);
}


async function postConfigChanges() {
    const obj = {};

    for (id of inputIds) {
        const input = getById(id);
        if (isInputDifferent(id)) {
            if (input.type === "checkbox") {
                obj[id] = input.checked;
            } else {
                obj[id] = Number(input.value);
            }
        }
    }

    obj.token = t;

    //console.log(obj);
    const response = await request("server/config", null, obj, "POST");

    console.log(response);

    if (!(response === null)) {
        for (id of inputIds) {
            const input = getById(id);
            if (input.type === "checkbox") {
                configObj[id] = input.checked;
            } else {
                configObj[id] = input.value;
            }

            getById(id + "Marker").innerHTML = "";
        }

        revertConfigChanges();
    }
}



function isInputDifferent(inputId) {
    const input = getById(inputId);
    const marker = getById(inputId + "Marker");

    if (input.type === "checkbox") {
        return !(input.checked === configObj[inputId]);
    } else {
        return !(input.value === configObj[inputId] + "");
    }
}


function revertConfigChanges() {
    for (id of inputIds) {
        const input = getById(id);
        if (input.type === "checkbox") {
            input.checked = configObj[id];
        } else {
            getById(id).value = configObj[id];
        }
        getById(id + "Marker").innerHTML = "";
    }

    liveNumberOfChangedInputs = 0;
    getById("postConfigButton").disabled = true;
    getById("discardConfigChangesButton").disabled = true;
}