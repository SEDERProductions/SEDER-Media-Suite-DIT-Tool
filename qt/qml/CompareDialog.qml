import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.platform as Platform

// Compare/verify the current Source folder against another folder (e.g. an
// already-offloaded backup) by path+size, modified time, or checksum. Lists
// only the differences; the matched count is shown in the summary.
Dialog {
    id: compareDialog
    modal: true
    title: "Compare / Verify"
    standardButtons: Dialog.Close

    property string destPath: ""
    property var report: ({ "summary": { "matched": 0, "differing": 0, "missing": 0, "extra": 0 }, "entries": [] })
    readonly property var modeKeys: ["PATHSIZE", "MTIME", "CHECKSUM"]

    readonly property bool dark: themeController.dark
    readonly property color bg: dark ? "#12110f" : "#ece6d9"
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color green: dark ? "#4cab7e" : "#1f7a4d"
    readonly property color warn: dark ? "#c99746" : "#9a6a16"
    readonly property color bad: dark ? "#d25645" : "#b43a1f"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string mono: "Menlo, Consolas, monospace"

    function urlToLocal(u) {
        const s = u.toString()
        if (s.startsWith("file:///")) {
            const stripped = s.substring(8)
            return (stripped.length >= 3 && stripped.charAt(1) === ":")
                ? decodeURIComponent(stripped) : "/" + decodeURIComponent(stripped)
        } else if (s.startsWith("file://")) {
            return decodeURIComponent(s.substring(7))
        }
        return decodeURIComponent(s)
    }

    function statusColor(status) {
        if (status === "extra_in_dest") return warn
        return bad
    }
    function statusLabel(status) {
        switch (status) {
        case "missing_in_dest": return "missing"
        case "extra_in_dest": return "extra"
        case "size_mismatch": return "size"
        case "mtime_mismatch": return "mtime"
        case "checksum_mismatch": return "checksum"
        default: return status
        }
    }

    background: Rectangle {
        color: panel
        border.color: line
        radius: 4
    }

    Platform.FolderDialog {
        id: destFolderDialog
        title: "Choose folder to compare against"
        onAccepted: compareDialog.destPath = compareDialog.urlToLocal(folder)
    }

    Connections {
        target: appController
        function onCompareStateChanged() {
            if (!appController.comparing && appController.compareResultJson.length > 0) {
                try {
                    compareDialog.report = JSON.parse(appController.compareResultJson)
                } catch (e) {
                    compareDialog.report = { "summary": { "matched": 0, "differing": 0, "missing": 0, "extra": 0 }, "entries": [] }
                }
            }
        }
    }

    ColumnLayout {
        width: 580
        spacing: 12

        MetaLabel { text: "Source" }
        Text {
            Layout.fillWidth: true
            text: appController.sourcePath.length > 0 ? appController.sourcePath : "Choose a Source folder first"
            color: appController.sourcePath.length > 0 ? ink : faint
            font.family: mono
            font.pixelSize: 11
            elide: Text.ElideMiddle
        }

        MetaLabel { text: "Compare against" }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text {
                Layout.fillWidth: true
                text: compareDialog.destPath.length > 0 ? compareDialog.destPath : "No folder chosen"
                color: compareDialog.destPath.length > 0 ? ink : faint
                font.family: mono
                font.pixelSize: 11
                elide: Text.ElideMiddle
            }
            QuietButton {
                text: "Choose…"
                onClicked: destFolderDialog.open()
                Accessible.name: "Choose folder to compare against"
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            FieldLabel { text: "Mode" }
            StyledComboBox {
                id: modeCombo
                Layout.preferredWidth: 200
                model: ["Path & Size", "Modified Time", "Checksum"]
                currentIndex: 0
                Accessible.name: "Comparison mode"
            }
            Item { Layout.fillWidth: true }
            QuietButton {
                text: appController.comparing ? "Comparing…" : "Compare"
                variant: "primary"
                enabled: !appController.comparing
                         && appController.sourcePath.length > 0
                         && compareDialog.destPath.length > 0
                onClicked: appController.runCompare(compareDialog.destPath,
                                                    compareDialog.modeKeys[modeCombo.currentIndex])
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: line }

        // Summary chips.
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Repeater {
                model: [
                    { "label": "Matched", "value": compareDialog.report.summary.matched, "tone": green },
                    { "label": "Differing", "value": compareDialog.report.summary.differing, "tone": bad },
                    { "label": "Missing", "value": compareDialog.report.summary.missing, "tone": bad },
                    { "label": "Extra", "value": compareDialog.report.summary.extra, "tone": warn }
                ]
                Rectangle {
                    Layout.fillWidth: true
                    height: 44
                    radius: 4
                    color: panelAlt
                    border.color: line
                    Column {
                        anchors.centerIn: parent
                        spacing: 2
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: modelData.value
                            color: modelData.tone
                            font.family: mono
                            font.pixelSize: 16
                            font.bold: true
                        }
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: modelData.label
                            color: faint
                            font.family: sans
                            font.pixelSize: 10
                        }
                    }
                }
            }
        }

        Text {
            visible: compareDialog.report.entries.length === 0 && !appController.comparing
            text: (compareDialog.report.summary.matched > 0
                   || compareDialog.report.summary.differing > 0
                   || compareDialog.report.summary.missing > 0
                   || compareDialog.report.summary.extra > 0)
                  ? "✓ No differences — every file matched."
                  : "Choose a folder and press Compare."
            color: muted
            font.family: sans
            font.pixelSize: 12
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 240
            visible: compareDialog.report.entries.length > 0
            color: bg
            border.color: line
            radius: 4
            ListView {
                anchors.fill: parent
                anchors.margins: 8
                clip: true
                model: compareDialog.report.entries
                spacing: 2
                ScrollBar.vertical: ScrollBar {}
                delegate: RowLayout {
                    width: ListView.view.width
                    spacing: 8
                    Rectangle {
                        Layout.preferredWidth: 70
                        height: 16
                        radius: 8
                        color: "transparent"
                        border.color: compareDialog.statusColor(modelData.status)
                        Text {
                            anchors.centerIn: parent
                            text: compareDialog.statusLabel(modelData.status)
                            color: compareDialog.statusColor(modelData.status)
                            font.family: compareDialog.mono
                            font.pixelSize: 9
                            font.bold: true
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        text: modelData.rel_path
                        color: ink
                        font.family: compareDialog.mono
                        font.pixelSize: 11
                        elide: Text.ElideMiddle
                    }
                }
            }
        }
    }
}
