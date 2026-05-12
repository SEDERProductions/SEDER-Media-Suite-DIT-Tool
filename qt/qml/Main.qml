import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: root
    width: 1360
    height: 860
    minimumWidth: 1080
    minimumHeight: 720
    visible: true
    title: "SEDER Media Suite DIT"

    readonly property bool dark: themeController.dark
    readonly property color bg: dark ? "#12110f" : "#ece6d9"
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property color green: dark ? "#4cab7e" : "#1f7a4d"
    readonly property color warn: dark ? "#c99746" : "#9a6a16"
    readonly property color bad: dark ? "#d25645" : "#b43a1f"
    readonly property string mono: "Menlo, Consolas, monospace"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    property bool metadataExpanded: false
    property bool logAutoScrollEnabled: true
    readonly property real scaleFactor: Math.max(1.0, Math.min(Screen.devicePixelRatio, 2.0))
    readonly property int spaceSmall: Math.round(6 * scaleFactor)
    readonly property int spaceMedium: Math.round(10 * scaleFactor)
    readonly property int spaceLarge: Math.round(16 * scaleFactor)
    readonly property int fontSmall: Math.round(10 * scaleFactor)
    readonly property int fontMedium: Math.round(12 * scaleFactor)
    readonly property int fontLarge: Math.round(14 * scaleFactor)
    readonly property int fontTitle: Math.round(24 * scaleFactor)
    readonly property int fieldHeight: Math.round(32 * scaleFactor)
    readonly property int actionHeight: Math.round(38 * scaleFactor)
    readonly property int compactPanelHeight: Math.round(58 * scaleFactor)

    color: bg

    function toneColor(tone) {
        if (tone === "good") return green
        if (tone === "warn") return warn
        if (tone === "bad") return bad
        return faint
    }

    function logSeverity(entry) {
        const m = entry.match(/\[[0-9]{2}:[0-9]{2}:[0-9]{2}\]\s+\[(INFO|WARN|ERROR)\]/)
        return m ? m[1] : "INFO"
    }

    function logMessage(entry) {
        return entry.replace(/^\[[0-9]{2}:[0-9]{2}:[0-9]{2}\]\s+\[(INFO|WARN|ERROR)\]\s*/, "")
    }

    function logTimestamp(entry) {
        const m = entry.match(/^\[([0-9]{2}:[0-9]{2}:[0-9]{2})\]/)
        return m ? m[1] : ""
    }

    function destinationStateLabel(state) {
        switch(state) {
        case 0: return "○ Pending"
        case 1: return "… Scanning"
        case 2: return "… Copying"
        case 3: return "… Verifying"
        case 4: return "✓ Complete"
        case 5: return "✕ Failed"
        case 6: return "✕ Cancelled"
        default: return "○ Pending"
        }
    }

    function destinationStateA11yLabel(state) {
        switch(state) {
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



    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.fillHeight: true
            Layout.preferredWidth: 366
            color: panel
            border.color: line

            ScrollView {
                anchors.fill: parent
                clip: true
                ColumnLayout {
                    width: parent.width
                    spacing: 14
                    anchors.margins: 16

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4
                        Text {
                            text: "SEDER Media Suite DIT"
                            color: ink
                            font.family: root.sans
                            font.pixelSize: 24
                            font.bold: true
                        }
                        MetaLabel { text: "Vol. 04 / Offload Verification" }
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: line }

                    MetaLabel { text: "01 / Source" }
                    PathPicker {
                        label: "Source folder"
                        path: appController.sourcePath
                        busy: appController.busy
                        onPick: appController.chooseSourceFolder()
                    }

                    MetaLabel { text: "02 / Destinations" }
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 6
                        Repeater {
                            model: appController.destinationModel
                            Rectangle {
                                Layout.fillWidth: true
                                height: 60
                                color: panelAlt
                                border.color: line
                                radius: 4
                                RowLayout {
                                    anchors.fill: parent
                                    anchors.margins: 8
                                    spacing: 8
                                    ColumnLayout {
                                        Layout.fillWidth: true
                                        spacing: 2
                                        Text {
                                            text: model.label || "Destination"
                                            color: ink
                                            font.family: root.sans
                                            font.pixelSize: 12
                                            font.bold: true
                                        }
                                        Text {
                                            text: model.path
                                            color: muted
                                            font.family: root.mono
                                            font.pixelSize: 10
                                            elide: Text.ElideMiddle
                                            Layout.fillWidth: true
                                        }
                                        Text {
                                            visible: model.error && model.error.length > 0
                                            text: model.error
                                            color: bad
                                            font.family: root.sans
                                            font.pixelSize: 10
                                            elide: Text.ElideRight
                                            Layout.fillWidth: true
                                        }
                                    }
                                    ColumnLayout {
                                        Layout.preferredWidth: 70
                                        Text {
                                            text: root.destinationStateLabel(model.state)
                                            color: model.state === 4 ? green : (model.state === 5 ? bad : (model.state === 6 ? faint : warn))
                                            font.family: root.mono
                                            font.pixelSize: 10
                                            font.bold: model.state === 4 || model.state === 5
                                            horizontalAlignment: Text.AlignRight
                                            Layout.fillWidth: true
                                            Accessible.name: "Destination status: " + root.destinationStateA11yLabel(model.state)
                                        }
                                        StyledProgressBar {
                                            Layout.fillWidth: true
                                            from: 0
                                            to: 1
                                            value: model.progress
                                            visible: model.state === 2 || model.state === 3
                                        }
                                    }
                                    QuietButton {
                                        text: "×"
                                        Layout.preferredWidth: 28
                                        variant: "danger"
                                        enabled: !appController.busy
                                        onClicked: appController.removeDestination(index)
                                    }
                                }
                            }
                        }
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        text: "+ Add Destination"
                        enabled: !appController.busy
                        onClicked: appController.addDestinationFolder()
                    }

                    MetaLabel { text: "03 / Options" }
                    StyledComboBox {
                        Layout.fillWidth: true
                        model: ["Verify after copy", "Copy only"]
                        currentIndex: appController.verifyAfterCopy ? 0 : 1
                        enabled: !appController.busy
                        onActivated: appController.verifyAfterCopy = (index === 0)
                    }
                    Text {
                        Layout.fillWidth: true
                        text: "MHL export requires Verify after copy."
                        color: muted
                        font.family: root.sans
                        font.pixelSize: 11
                        wrapMode: Text.WordWrap
                    }
                    StyledCheckBox {
                        text: "Skip existing files"
                        checked: appController.skipExisting
                        enabled: !appController.busy
                        onToggled: appController.skipExisting = checked
                    }
                    StyledCheckBox {
                        text: "Generate report after offload"
                        checked: appController.generateReport
                        enabled: !appController.busy
                        onToggled: appController.generateReport = checked
                    }
                    StyledCheckBox {
                        text: "Ignore hidden/system files"
                        checked: appController.ignoreHiddenSystem
                        enabled: !appController.busy
                        onToggled: appController.ignoreHiddenSystem = checked
                        font.family: root.sans
                        font.pixelSize: 12
                    }
                    FieldLabel { text: "Ignore patterns" }
                    TextArea {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 60
                        text: appController.ignorePatterns
                        enabled: !appController.busy
                        color: ink
                        placeholderTextColor: faint
                        font.family: root.mono
                        font.pixelSize: 11
                        wrapMode: TextArea.Wrap
                        onTextChanged: appController.ignorePatterns = text
                        background: Rectangle { color: panelAlt; border.color: line; radius: 4 }
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: line }
                    QuietButton {
                        Layout.fillWidth: true
                        text: "Sync Destinations to Source"
                        enabled: !appController.busy && appController.destinationModel.count > 0 && appController.sourcePath.length > 0
                        onClicked: appController.syncDestinationPaths()
                        ToolTip.visible: hovered
                        ToolTip.text: "Replace last path component of all destinations with source folder name"
                    }

                    QuietButton {
                        Layout.fillWidth: true
                        height: 38
                        text: appController.busy ? "Offloading..." : "Start Offload"
                        variant: "primary"
                        enabled: !appController.busy && appController.destinationModel.count > 0
                        onClicked: appController.startOffload()
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        height: 38
                        text: "Cancel"
                        variant: "danger"
                        visible: appController.busy
                        onClicked: appController.cancelOffload()
                    }

                    Rectangle { Layout.fillWidth: true; height: 1; color: line }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 2
                        RowLayout {
                            Layout.fillWidth: true
                            height: 28
                            spacing: 8
                            MetaLabel { text: "04 / Metadata" }
                            Item { Layout.fillWidth: true }
                            ToolButton {
                                id: metadataToggleButton
                                text: metadataExpanded ? "Collapse ▼" : "Expand ▶"
                                focusPolicy: Qt.StrongFocus
                                Accessible.name: metadataExpanded ? "Collapse metadata fields" : "Expand metadata fields"
                                Accessible.description: Accessible.name
                                ToolTip.visible: hovered
                                ToolTip.text: Accessible.name
                                onClicked: metadataExpanded = !metadataExpanded
                                Keys.onReturnPressed: {
                                    metadataExpanded = !metadataExpanded
                                    event.accepted = true
                                }
                                Keys.onEnterPressed: {
                                    metadataExpanded = !metadataExpanded
                                    event.accepted = true
                                }
                                Keys.onSpacePressed: {
                                    metadataExpanded = !metadataExpanded
                                    event.accepted = true
                                }
                                contentItem: Text {
                                    text: parent.text
                                    color: parent.down ? ink : faint
                                    font.family: root.mono
                                    font.pixelSize: 10
                                    horizontalAlignment: Text.AlignHCenter
                                    verticalAlignment: Text.AlignVCenter
                                }
                                background: Rectangle {
                                    radius: 4
                                    color: metadataToggleButton.down ? panelAlt : "transparent"
                                    border.color: metadataToggleButton.visualFocus ? line : "transparent"
                                }
                            }
                        }
                        ColumnLayout {
                            Layout.fillWidth: true
                            visible: metadataExpanded
                            spacing: 8
                            FieldLabel { text: "Project name" }
                            DenseTextField {
                                Layout.fillWidth: true
                                text: appController.projectName
                                enabled: !appController.busy
                                maximumLength: 256
                                onTextChanged: appController.projectName = text
                            }
                            FieldLabel { text: "Shoot date" }
                            DenseTextField {
                                Layout.fillWidth: true
                                text: appController.shootDate
                                placeholderText: "YYYY-MM-DD"
                                enabled: !appController.busy
                                maximumLength: 10
                                onTextChanged: appController.shootDate = text
                            }
                            RowLayout {
                                Layout.fillWidth: true
                                spacing: 8
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    FieldLabel { text: "Card name" }
                                    DenseTextField {
                                        Layout.fillWidth: true
                                        text: appController.cardName
                                        enabled: !appController.busy
                                        maximumLength: 64
                                        onTextChanged: appController.cardName = text
                                    }
                                }
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    FieldLabel { text: "Camera ID" }
                                    DenseTextField {
                                        Layout.fillWidth: true
                                        text: appController.cameraId
                                        enabled: !appController.busy
                                        maximumLength: 64
                                        onTextChanged: appController.cameraId = text
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        ColumnLayout {
            Layout.fillHeight: true
            Layout.fillWidth: true
            spacing: 0

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 94
                color: bg
                border.color: line
                RowLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 10
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 58
                        color: panelAlt
                        border.color: line
                        radius: 4
                        Column {
                            anchors.fill: parent
                            anchors.margins: 9
                            spacing: 5
                            MetaLabel { text: "Files" }
                            Text {
                                text: appController.totalFiles
                                color: toneColor("neutral")
                                font.family: root.mono
                                font.pixelSize: 18
                                font.bold: true
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 58
                        color: panelAlt
                        border.color: line
                        radius: 4
                        Column {
                            anchors.fill: parent
                            anchors.margins: 9
                            spacing: 5
                            MetaLabel { text: "Size" }
                            Text {
                                text: appController.formatBytes(appController.totalSize)
                                color: toneColor("neutral")
                                font.family: root.mono
                                font.pixelSize: 18
                                font.bold: true
                            }
                        }
                    }
                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 58
                        color: appController.finalStatus === "PASS"
                               ? (dark ? "#1a2a1f" : "#e0f0e6")
                               : (appController.finalStatus === "COPIED (UNVERIFIED)"
                                  ? (dark ? "#2c271b" : "#f4ecd8")
                                  : panelAlt)
                        border.color: appController.finalStatus === "PASS"
                                      ? green
                                      : (appController.finalStatus === "COPIED (UNVERIFIED)" ? warn : line)
                        radius: 4
                        visible: appController.canExport
                        Column {
                            anchors.fill: parent
                            anchors.margins: 9
                            spacing: 2
                            MetaLabel { text: "Status" }
                            Text {
                                text: appController.finalStatus === "PASS"
                                      ? "✓ PASS"
                                      : (appController.finalStatus === "COPIED (UNVERIFIED)"
                                         ? "⚠ COPIED (UNVERIFIED)"
                                         : "✕ FAIL")
                                color: appController.finalStatus === "PASS"
                                       ? green
                                       : (appController.finalStatus === "COPIED (UNVERIFIED)" ? warn : bad)
                                font.family: root.mono
                                font.pixelSize: 18
                                font.bold: true
                            }
                            Text {
                                text: appController.finalStatus === "PASS"
                                      ? "All destination verifications passed"
                                      : (appController.finalStatus === "COPIED (UNVERIFIED)"
                                         ? "Files copied but not verified"
                                         : "One or more destination verifications failed")
                                color: muted
                                font.family: root.sans
                                font.pixelSize: 10
                                font.bold: true
                                elide: Text.ElideRight
                                width: parent.width
                            }
                        }
                        Accessible.name: appController.finalStatus === "PASS"
                                         ? "Pass status. All destination verifications passed."
                                         : (appController.finalStatus === "COPIED (UNVERIFIED)"
                                            ? "Copied unverified status. Files copied but not verified."
                                            : "Fail status. One or more destination verifications failed.")
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 50
                color: panel
                border.color: line
                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: 16
                    anchors.rightMargin: 16
                    spacing: 8
                    Item { Layout.fillWidth: true }
                    QuietButton { text: "TXT"; enabled: appController.canExport && !appController.busy; onClicked: appController.exportTxt() }
                    QuietButton { text: "CSV"; enabled: appController.canExport && !appController.busy; onClicked: appController.exportCsv() }
                    QuietButton {
                        text: "MHL"
                        enabled: appController.canExportMhl && !appController.busy
                        visible: appController.canExportMhl || appController.canExport
                        onClicked: appController.exportMhl()
                        ToolTip.visible: hovered && !enabled
                        ToolTip.text: "Requires Verify after copy"
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: bg
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 16
                    spacing: 0
                    Rectangle {
                        visible: !appController.busy && !appController.canExport
                        Layout.alignment: Qt.AlignCenter
                        Layout.preferredWidth: 480
                        Layout.preferredHeight: 220
                        color: "transparent"
                        Column {
                            anchors.centerIn: parent
                            spacing: 16
                            Text {
                                text: "Ready For Offload"
                                color: ink
                                font.family: root.sans
                                font.pixelSize: 24
                                font.bold: true
                                horizontalAlignment: Text.AlignHCenter
                                width: 480
                            }
                            Column {
                                anchors.horizontalCenter: parent.horizontalCenter
                                spacing: 8
                                Repeater {
                                    model: [
                                        "1. Choose a Source folder",
                                        "2. Add one or more Destinations",
                                        "3. Configure Options",
                                        "4. Click Start Offload"
                                    ]
                                    Text {
                                        text: modelData
                                        color: muted
                                        font.family: root.sans
                                        font.pixelSize: 14
                                        horizontalAlignment: Text.AlignHCenter
                                        width: 480
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 128
                color: panel
                border.color: line
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 12
                    spacing: 8
                    RowLayout {
                        Layout.fillWidth: true
                        MetaLabel { text: appController.busy ? "Working" : "Status" }
                        Text {
                            Layout.fillWidth: true
                            text: appController.statusText
                            color: muted
                            font.family: root.sans
                            font.pixelSize: 12
                            elide: Text.ElideRight
                        }
                        StyledProgressBar {
                            Layout.preferredWidth: 180
                            from: 0
                            to: 1
                            value: appController.overallProgress
                            indeterminate: appController.busy && appController.statusText === "Scanning source..." && appController.overallProgress <= 0
                            visible: appController.busy || appController.overallProgress > 0
                        }
                        StyledComboBox {
                            Layout.preferredWidth: 70
                            model: ["Auto", "Light", "Dark"]
                            currentIndex: themeController.preference === "dark" ? 2 : (themeController.preference === "light" ? 1 : 0)
                            onActivated: {
                                const map = ["system", "light", "dark"]
                                themeController.preference = map[index]
                            }
                            font.pixelSize: 10
                        }
                        QuietButton {
                            text: "Copy Log"
                            enabled: appController.logLines.length > 0
                            onClicked: appController.copyLog()
                        }
                        QuietButton {
                            text: "Clear Log"
                            enabled: appController.logLines.length > 0
                            onClicked: appController.clearLog()
                        }
                    }
                    Text {
                        text: appController.statusText === "Scanning source..."
                              ? "Indexing source files and checksums..."
                              : appController.currentFile
                        color: faint
                        font.family: root.mono
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                        Layout.fillWidth: true
                        visible: appController.busy && (appController.currentFile.length > 0 || appController.statusText === "Scanning source...")
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: line }
                    ListView {
                        id: logListView
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        model: appController.logLines
                        spacing: 2
                        onCountChanged: {
                            if (root.logAutoScrollEnabled && count > 0) {
                                positionViewAtEnd()
                            }
                        }
                        onContentYChanged: {
                            const atBottom = (contentY + height) >= (contentHeight - 8)
                            if (!appController.busy) {
                                root.logAutoScrollEnabled = true
                            } else {
                                root.logAutoScrollEnabled = atBottom
                            }
                        }
                        delegate: RowLayout {
                            width: ListView.view.width
                            spacing: 8
                            property string severity: root.logSeverity(modelData)
                            property color sevColor: severity === "ERROR" ? bad : (severity === "WARN" ? warn : muted)
                            Text {
                                text: severity === "ERROR" ? "⛔" : (severity === "WARN" ? "⚠" : "•")
                                color: parent.sevColor
                                font.family: root.sans
                                font.pixelSize: 10
                            }
                            Text {
                                text: root.logTimestamp(modelData)
                                color: faint
                                font.family: root.mono
                                font.pixelSize: 10
                            }
                            Text {
                                Layout.fillWidth: true
                                text: root.logMessage(modelData)
                                color: parent.sevColor
                                font.family: root.mono
                                font.pixelSize: 10
                                elide: Text.ElideRight
                            }
                        }
                        ScrollBar.vertical: ScrollBar {}
                    }
                }
            }
        }
    }
}

