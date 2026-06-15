.pragma library

// Activity-log line parsing — moved out of Main.qml so views share it.
function logSeverity(entry) {
    var m = entry.match(/\[[0-9]{2}:[0-9]{2}:[0-9]{2}\]\s+\[(INFO|WARN|ERROR)\]/)
    return m ? m[1] : "INFO"
}

function logMessage(entry) {
    return entry.replace(/^\[[0-9]{2}:[0-9]{2}:[0-9]{2}\]\s+\[(INFO|WARN|ERROR)\]\s*/, "")
}

function logTimestamp(entry) {
    var m = entry.match(/^\[([0-9]{2}:[0-9]{2}:[0-9]{2})\]/)
    return m ? m[1] : ""
}

function severityIcon(sev) {
    if (sev === "ERROR") return "octagon"
    if (sev === "WARN") return "alert"
    return "dot"
}

// Destination state enum (mirror of DestinationState in src/offload/mod.rs):
// 0 Pending, 1 Scanning, 2 Copying, 3 Verifying, 4 Complete, 5 Failed, 6 Cancelled.
function destStateLabel(state) {
    switch (state) {
    case 0: return "Pending"
    case 1: return "Scanning"
    case 2: return "Copying"
    case 3: return "Verifying"
    case 4: return "Complete"
    case 5: return "Failed"
    case 6: return "Cancelled"
    default: return "Pending"
    }
}

function destStateIcon(state) {
    switch (state) {
    case 0: return "circle"
    case 1: return "search"
    case 2: return "activity"
    case 3: return "refresh"
    case 4: return "check-circle"
    case 5: return "x-circle"
    case 6: return "slash-circle"
    default: return "circle"
    }
}

function destStateTone(state) {
    switch (state) {
    case 4: return "good"
    case 5: return "bad"
    case 6: return "neutral"
    case 1:
    case 2:
    case 3: return "warn"
    default: return "neutral"
    }
}
