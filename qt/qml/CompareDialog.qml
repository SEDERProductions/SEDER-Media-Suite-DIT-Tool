import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Qt.labs.platform as Platform

Dialog {
    id: compareDialog
    modal: true
    title: "Compare / Verify Folders"
    standardButtons: Dialog.Close
    width: 640
    height: 520

    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color green: dark ? "#4cab7e" : "#1f7a4d"
    readonly property color bad: dark ? "#d25645" : "#b43a1f"
    readonly property color warn: dark ? "#c99746" : "#9a6a16"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string mono: "Menlo, Consolas, monospace"

    // Holds the last parsed compare report (avoid shadowing Dialog.result)
    property var report: null
    property string destFolder: ""

    background: Rectangle {
        color: panel
        border.color: dark ? "#3a352e" : "#d6cfbe"
        radius: 4
    }

    Platform.FolderDialog {
        id: folderPicker
        title: "Choose folder to compare against source"
        onAccepted: compareDialog.destFolder = folderPicker.folder.toString().replace(/^file:\/\//, "")
    }

    Connections {
        target: appController
        function onCompareStateChanged() {
            if (!appController.comparing && appController.compareResultJson.length > 0) {
                try {
                    compareDialog.report = JSON.parse(appController.compareResultJson)
                } catch (e) {
                    compareDialog.report = null
                }
            }
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        // Folder picker row
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text {
                text: compareDialog.destFolder.length > 0
                      ? compareDialog.destFolder
                      : "No folder selected"
                color: compareDialog.destFolder.length > 0 ? ink : faint
                font.family: mono
                font.pixelSize: 11
                Layout.fillWidth: true
                elide: Text.ElideMiddle
            }
            Button {
                text: "Choose…"
                enabled: !appController.comparing
                Accessible.name: "Choose comparison folder"
                onClicked: folderPicker.open()
                contentItem: Text {
                    text: parent.text
                    color: ink
                    font.family: compareDialog.sans
                    font.pixelSize: 12
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: parent.down ? panelAlt : panel
                    border.color: line
                    radius: 4
                }
            }
        }

        // Mode selector
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Text {
                text: "Compare mode:"
                color: muted
                font.family: sans
                font.pixelSize: 12
            }
            ComboBox {
                id: modeBox
                model: ["Path & Size", "Modified Time", "Checksum"]
                Layout.preferredWidth: 160
                enabled: !appController.comparing
                font.family: sans
                font.pixelSize: 12
                contentItem: Text {
                    leftPadding: 8
                    text: modeBox.displayText
                    color: ink
                    font: modeBox.font
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: panelAlt
                    border.color: line
                    radius: 4
                }
                delegate: ItemDelegate {
                    width: modeBox.width
                    contentItem: Text {
                        leftPadding: 8
                        text: modelData
                        color: ink
                        font.family: sans
                        font.pixelSize: 12
                        verticalAlignment: Text.AlignVCenter
                    }
                    background: Rectangle {
                        color: hovered ? panel : panelAlt
                    }
                }
                popup: Popup {
                    y: modeBox.height + 2
                    width: modeBox.width
                    padding: 1
                    contentItem: ListView {
                        clip: true
                        implicitHeight: contentHeight
                        model: modeBox.delegateModel
                        ScrollBar.vertical: ScrollBar {}
                    }
                    background: Rectangle {
                        color: panelAlt
                        border.color: line
                        radius: 4
                    }
                }
            }
            Item { Layout.fillWidth: true }
            Button {
                text: appController.comparing ? "Running…" : "Run Compare"
                enabled: !appController.comparing
                    && compareDialog.destFolder.length > 0
                    && appController.sourcePath.length > 0
                Accessible.name: "Run compare"
                onClicked: {
                    const modeMap = ["path_size", "mtime", "checksum"]
                    appController.runCompare(compareDialog.destFolder, modeMap[modeBox.currentIndex])
                }
                contentItem: Text {
                    text: parent.text
                    color: parent.enabled ? green : faint
                    font.family: compareDialog.sans
                    font.pixelSize: 12
                    font.bold: true
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }
                background: Rectangle {
                    color: parent.down ? panelAlt : panel
                    border.color: parent.enabled ? green : line
                    radius: 4
                }
            }
        }

        // Summary chips
        Flow {
            Layout.fillWidth: true
            spacing: 8
            visible: compareDialog.report !== null

            Repeater {
                model: compareDialog.report ? [
                    { label: "Matched", value: compareDialog.report.matched, color: green },
                    { label: "Differing", value: compareDialog.report.differing, color: bad },
                    { label: "Missing in dest", value: compareDialog.report.missing_in_dest, color: warn },
                    { label: "Extra in dest", value: compareDialog.report.extra_in_dest, color: faint }
                ] : []
                Rectangle {
                    height: 28
                    width: chipLabel.implicitWidth + 24
                    radius: 14
                    color: panelAlt
                    border.color: modelData.color
                    Text {
                        id: chipLabel
                        anchors.centerIn: parent
                        text: modelData.label + ": " + modelData.value
                        color: modelData.color
                        font.family: compareDialog.sans
                        font.pixelSize: 11
                        font.bold: true
                    }
                }
            }
        }

        // Differences list
        Rectangle {
            Layout.fillWidth: true
            Layout.fillHeight: true
            color: panelAlt
            border.color: line
            radius: 4
            clip: true

            Text {
                anchors.centerIn: parent
                visible: !appController.comparing && compareDialog.report === null
                text: "Select a folder and run compare to see differences."
                color: faint
                font.family: sans
                font.pixelSize: 12
            }

            BusyIndicator {
                anchors.centerIn: parent
                visible: appController.comparing
                running: appController.comparing
            }

            ListView {
                anchors.fill: parent
                anchors.margins: 4
                visible: compareDialog.report !== null && compareDialog.report.entries !== undefined
                model: compareDialog.report ? compareDialog.report.entries : []
                clip: true
                ScrollBar.vertical: ScrollBar {}

                delegate: RowLayout {
                    width: ListView.view.width
                    height: 26
                    spacing: 8
                    property color statusColor: {
                        switch (modelData.status) {
                        case "missing_in_dest":  return warn
                        case "extra_in_dest":    return faint
                        case "size_mismatch":    return bad
                        case "mtime_mismatch":   return warn
                        case "checksum_mismatch": return bad
                        case "error":            return bad
                        default: return faint
                        }
                    }
                    Text {
                        Layout.preferredWidth: 110
                        text: modelData.status.replace(/_/g, " ")
                        color: parent.statusColor
                        font.family: mono
                        font.pixelSize: 10
                        elide: Text.ElideRight
                    }
                    Text {
                        Layout.fillWidth: true
                        text: modelData.path
                        color: ink
                        font.family: mono
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                    }
                    Text {
                        visible: modelData.detail !== undefined
                        text: modelData.detail || ""
                        color: faint
                        font.family: mono
                        font.pixelSize: 9
                        Layout.preferredWidth: 120
                        elide: Text.ElideRight
                    }
                }
            }
        }
    }
}
