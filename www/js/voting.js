//
//
//
//
//


window.onresize = resizeEmbeds;


//
//
// On-load
//
//


var givenCategory;
var desiredCategory = 0;
var votingStage = 0;
var currentVotingStageP = getById("stage1TimeP");
var currentDeadline = -1;

var videos = [];

const updateTimeFunc = setInterval(updateDeadlineCountdown, 1000);

initiateNewVote();


//
//
// General-ish Boilerplate
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
    if ((queries === null)) { url = endpoint } else { url = endpoint + "?" + (new URLSearchParams(queries)).toString(); }

    let status = "";
    let fallbackText = "";

    let json = await window.fetch(url, options).then((response) => {
        return [response.status, response.statusText, response.json()];
    }).then((responseArr) => {
        status = responseArr[0];
        fallbackText = responseArr[1];
        return responseArr[2];
    });


    if (status === 200) {
        clearErrField();
        return json;
    } else {
        if (status === 423) {
            getById("stageBox").classList.add("hidden");
            getById("categoryHeaderBox").classList.add("hidden");
            getById("categorySelectionBox").classList.add("hidden");
            getById("categoryInfoBox").classList.add("hidden");
            getById("votingBox").classList.add("hidden");
            getById("errField").classList.add("hidden");
            getById("infoAndHelpBox").classList.add("hidden");
        
            getById("mainErrorPaneHeader").innerHTML = "Voting closed";
            getById("mainErrorPaneText").innerHTML = "Voting is currently not allowed at this time";
        } else {
            console.log(json);
            if (Object.hasOwn(json, "message")) {
                getById("errFieldP").innerHTML = "Error: " + json.message;
            } else {
                getById("errFieldP").innerHTML = "Error: " + fallbackText;
            }
        }
        return null;
    }
}


function getById(id) {
    return document.getElementById(id);
}


function clearErrField() {
    getById("errFieldP").innerHTML = "";
}



//
//
// Local storage
//
//

function swapTheme() {
    let themeToggleIcon = getById("themeToggleIcon");
    if(document.documentElement.getAttribute('data-theme') === 'dark') {
        document.documentElement.setAttribute('data-theme', 'light');
        localStorage.setItem('isDarkMode', false);
        //themeToggleIcon.classList.remove('ph-sun');
        //themeToggleIcon.classList.add('ph-moon-stars');
    } else {
        document.documentElement.setAttribute('data-theme', 'dark');
        localStorage.setItem('isDarkMode', true);
        //themeToggleIcon.classList.remove('ph-moon-stars');
        //themeToggleIcon.classList.add('ph-sun');
    }
}

//
//
// SortableJS
//
//


new Sortable(rankedListBox, {
    handle: '.ph-dots-six-vertical',
    animation: 150,

    onEnd: function (evt) {
        let ranks = ['1st', '2nd', '3rd', '4th', '5th'];
        let counter = 0;
        for (rankBox of evt.to.children) {
            rankBox.querySelector('#rankLabel').innerHTML = ranks[counter];
            counter += 1;
            if (!rankBox.disabled) {
                toggleVotingButton("rankedVoteButton", true);
            }
        }
    }
});


//
//
// VOTING
//
//


async function initiateNewVote() {
    // Lock voting buttons
    toggleVotingButton("voteButtonLeft", false);
    toggleVotingButton("voteButtonRight", false);
    toggleVotingButton("rankedVoteButton", false);

    // Reset any potential voting box error message
    getById("votingBoxErrorBox").classList.add("hidden");
    getById("votingBlock2Size").classList.add("invisible");
    getById("votingBlock5Size").classList.add("invisible");

    // API call
    let response = await request("vote", { c: desiredCategory }, null, "GET");

    if (response === null) {
        return;
    }

    // Set variables
    givenCategory = response.c;
    votingStage = response.stage;
    currentDeadline = response.current_deadline;
    videos = response.videos;

    // Apply stage subtext
    updateStage();
    if (desiredCategory === 0) {
        if (givenCategory > 0) {
            let categoryLabels = ["Comedy", "Skill"];
            getById("givenCategoryP").innerHTML = "Given category: " + categoryLabels[givenCategory - 1];
        }
    } else {
        getById("givenCategoryP").innerHTML = "";
    }

    // Toggle category buttons
    if (response.limit_active) {
        toggleCategoryButton(0, false);
        if (desiredCategory === 0) {
            selectDesiredCategory(givenCategory, false);
            getById("givenCategoryP").innerHTML = "";
        }
    } else {
        toggleCategoryButton(0, true);
    }

    // Setup voting pane
    switch (videos.length) {
        case 2:
            // Swap from 
            getById("votingBlock2Size").classList.remove("invisible");
            getById("votingBlock5Size").classList.remove("invisible");
            getById("votingBlock2Size").classList.remove("hidden");
            getById("votingBlock5Size").classList.add("hidden");

            refreshEmbed("embedBoxLeft", "frameLeft", videos[0].youtube_id);
            refreshEmbed("embedBoxRight", "frameRight", videos[1].youtube_id);
            toggleVotingButton("voteButtonLeft", true);
            toggleVotingButton("voteButtonRight", true);
            break;
        case 5:
            getById("votingBlock2Size").classList.remove("invisible");
            getById("votingBlock5Size").classList.remove("invisible");
            getById("votingBlock5Size").classList.remove("hidden");
            getById("votingBlock2Size").classList.add("hidden");

            let i = 0;
            for (rankBox of getById("rankedListBox").children) {
                rankBox.classList.remove("selected");
                rankBox.children[1].innerHTML = (videos[i].u === null) ? "" : videos[i].u;
                rankBox.children[2].innerHTML = "Submission #" + videos[i].id
                rankBox.dataset.videoIndex = i;
                i += 1;
            }
            clearEmbed("rankedFrame");
            break;
        case 0:
            getById("votingBlock2Size").classList.add("invisible");
            getById("votingBlock5Size").classList.add("invisible");
            getById("votingBoxErrorBox").classList.remove("hidden");
            const header = getById("votingBoxErrorHeader");
            const text = getById("votingBoxErrorText");
            // Nothing was returned because the vote limit was hit for this category
            if (response.limit_active) {
                header.innerHTML = "Vote submitted";
                text.innerHTML = "This round only allows 1 vote per category";
            // Nothing was returned because there's nothing to vote on (ie the databases are empty)
            } else {   
                header.innerHTML = "Nothing";
                text.innerHTML = "There's nothing to vote on";
                toggleCategoryButton(0, false);
                toggleCategoryButton(1, false);
                toggleCategoryButton(2, false);
            }
            break;
        default:
            break;
    }
}


function submitRankedVote() {
    let vote = [];
    for (rankBox of getById("rankedListBox").children) {
        vote.push(videos[Number(rankBox.dataset.videoIndex)].id);
    }
    postVote(vote);
}


function submitVote(index) {
    vote = [videos[index].id, videos[Number(!index)].id];
    postVote(vote);
}



async function postVote(vote) {
    toggleVotingButton("voteButtonLeft", false);
    toggleVotingButton("voteButtonRight", false);
    toggleVotingButton("rankedVoteButton", false);
    let response = await request("vote", null, { vote: vote }, "POST");
    if (!(response === null)) {
        clearEmbed("frameLeft");
        clearEmbed("frameRight");
        clearEmbed("rankedFrame");
        initiateNewVote();   
    }
    toggleVotingButton("voteButtonLeft", true);
    toggleVotingButton("voteButtonRight", true);
    toggleVotingButton("rankedVoteButton", true);
}


//
//
// SECONDARY VOTING
//
//


function selectRankedVideo(videoIndex) {
    const clickedRankBox = getById("rankBox" + (videoIndex + 1));

    if (!(clickedRankBox.classList.contains("selected"))) {
        refreshEmbed("rankedEmbedBox", "rankedFrame", videos[videoIndex].youtube_id);

        for (rankBox of getById("rankedListBox").children) {
            rankBox.classList.remove("selected");
        }
        clickedRankBox.classList.add("selected");

        if (!clickedRankBox.disabled) {
            toggleVotingButton("rankedVoteButton", true);
        }
    }
}


function selectDesiredCategory(newDesiredCategory, requestNewVote) {
    if (!(desiredCategory === newDesiredCategory)) {
        for (let i = 0; i < 3; i++) {
            getById("category" + i + "Button").classList.remove("selected");
        }

        getById("category" + newDesiredCategory + "Button").classList.add("selected");

        desiredCategory = newDesiredCategory;

        if (requestNewVote) { initiateNewVote() };
    }
}


//
//
// UI TOGGLING
//
//


function toggleCategoryButton(desiredCategory, toggleTo) {
    getById("category" + desiredCategory + "Button").disabled = !toggleTo;
}


function toggleVotingButton(buttonId, toggleTo) {
    getById(buttonId).disabled = !toggleTo;
}


function toggleVotingBoxErrorState(toggleTo) {
    if (!toggleTo) {
        getById("votingBlock2Size").classList.remove("invisible");
        getById("votingBlock5Size").classList.remove("invisible");
        getById("votingBoxErrorBox").classList.add("hidden");
    } else {
        getById("votingBlock2Size").classList.add("invisible");
        getById("votingBlock5Size").classList.add("invisible");
        getById("votingBoxErrorBox").classList.remove("hidden");
    }
}


function swapHiddenBetween(toId, fromId) {
    getById(toId).classList.add("hidden");
    getById(fromId).classList.remove("hidden");
}


//
//
// EMBEDS
//
//


function refreshEmbed(embedBoxId, frameId, youtubeId) {
    // remove old frame
    let frame = getById(frameId);
    if (!(frame === null)) {
        frame.remove();
    }

    // create and insert new frame
    let embedBox = getById(embedBoxId);
    let rect = embedBox.getBoundingClientRect();

    let newFrame = document.createElement('iframe');
    newFrame.id = frameId;
    newFrame.width = rect['width'];
    newFrame.height = rect['height'];
    newFrame.src = "https://www.youtube.com/embed/" + youtubeId;
    newFrame.setAttribute("allowfullscreen", "");
    newFrame.setAttribute("allow", "autoplay; allowfullscreen");
    embedBox.appendChild(newFrame);
}


function clearEmbed(frameId) {
    let frame = getById(frameId);
    if (!(frame === null)) {
        frame.remove();
    }
}


function resizeEmbeds() {
    let resizeIframe = function(id, rect) {
        let el = getById(id);
        if (!(el === null)) {
            el.width = rect['width'];
            el.height = rect['height']; 
        }  
    }

    if (votingStage === 2) {
        let rect = getById("embedBoxLeft").getBoundingClientRect();
        resizeIframe("frameLeft", rect);
        resizeIframe("frameRight", rect);
    } else {
        let rect = getById("rankedEmbedBox").getBoundingClientRect();
        resizeIframe("rankedFrame", rect);
    }
}


//
//
// STAGE/PROGRESS BAR
//
//


function updateStage() {
    // Configure stage boxes
    let stageBoxes = ["stage1Box", "stage2Box", "stage3Box"];
    for (stageNot of stageBoxes) {
        let stageNotEl = getById(stageNot);
        stageNotEl.classList.remove("stageFlagActive");
        stageNotEl.classList.add("stageFlagDefault");
    }
    let stageIsEl = getById(stageBoxes[votingStage - 1]);
    stageIsEl.classList.add("stageFlagActive");
    stageIsEl.classList.remove("stageFlagDefault");

    let stagePs = ["stage1TimeP", "stage2TimeP", "stage3TimeP"];
    currentVotingStageP = getById(stagePs[votingStage - 1]);
}


function updateDeadlineCountdown() {
    if (currentDeadline >= 0) {
        // Get today's date and time
        const now = Math.round(new Date().getTime() / 1000);    
        // Find the distance between now and the countdown date
        const distance = currentDeadline - now;    
        // Time calculations for days, hours, minutes and seconds
        const days = Math.floor(distance / (60 * 60 * 24));
        const hours = Math.floor((distance % (60 * 60 * 24)) / (60 * 60));
        const minutes = Math.floor((distance % (60 * 60)) / (60));

        if (days > 0) {
            currentVotingStageP.innerHTML = days + "d " + hours + "h " + minutes + "m";
        } else {
            currentVotingStageP.innerHTML = hours + "h " + minutes + "m";
        }

        if (distance < 0) {
            currentVotingStageP.innerHTML = "0h 0m";
            clearInterval(updateTimeFunc);
        }
    }
}
