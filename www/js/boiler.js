//
//
//
//
//

function shoClipsFetch(endpoint, payload, method) {
    window.fetch(endpoint, {
        method: method,
        headers: {
            "Content-Type": "application/json",
        },
        data: JSON.stringify(payload)
    }).then((response) => {
        return [response.status, response.json()];
    });
}


function getById(id) {
    return document.getElementById(id);
}