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

    color: bg

    function toneColor(tone) {
        if (tone === "good") return green
        if (tone === "warn") return warn
        if (tone === "bad") return bad
        return faint
    }

    component MetaLabel: Text {
        color: faint
        font.family: root.mono
        font.pixelSize: 10
        font.capitalization: Font.AllUppercase
    }

    component FieldLabel: Text {
        color: muted
        font.family: root.sans
        font.pixelSize: 12
    }

    component DenseTextField: TextField {
        color: ink
        selectedTextColor: "#ffffff"
        selectionColor: red
        font.family: root.sans
        font.pixelSize: 13
        padding: 8
        placeholderTextColor: faint
        background: Rectangle {
            color: panelAlt
            border.color: line
            radius: 4
        }
    }

    component QuietButton: Button {
        property bool danger: false
        height: 32
        font.family: root.sans
        font.pixelSize: 12
        contentItem: Text {
            text: parent.text
            color: parent.danger ? "#ffffff" : ink
            font: parent.font
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        background: Rectangle {
            color: parent.danger ? red : panelAlt
            border.color: parent.danger ? red : line
            radius: 4
            opacity: parent.enabled ? 1 : 0.45
        }
    }

    component StyledComboBox: ComboBox {
        id: control
        font.family: root.sans
        font.pixelSize: 12
        contentItem: Text {
            leftPadding: 8
            rightPadding: 8
            text: control.displayText
            color: ink
            font: control.font
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }
        background: Rectangle {
            color: panelAlt
            border.color: line
            radius: 4
        }
        popup: Popup {
            y: control.height
            width: control.width
            padding: 1
            background: Rectangle {
                color: panelAlt
                border.color: line
                radius: 4
            }
            contentItem: ListView {
                clip: true
                implicitHeight: contentHeight
                model: control.model
                currentIndex: control.currentIndex
                delegate: ItemDelegate {
                    width: ListView.view.width
                    contentItem: Text {
                        text: modelData
                        color: ink
                        font: control.font
                        elide: Text.ElideRight
                        verticalAlignment: Text.AlignVCenter
                    }
                    highlighted: control.highlightedIndex === index
                    background: Rectangle {
                        color: highlighted ? red : "transparent"
                        radius: 2
                    }
                }
                ScrollBar.vertical: ScrollBar {}
            }
        }
    }

    component PathPicker: Rectangle {
        id: pathPickerRoot
        property string label: ""
        property string path: ""
        signal pick()
        Layout.fillWidth: true
        height: 68
        color: "transparent"
        ColumnLayout {
            anchors.fill: parent
            spacing: 6
            FieldLabel { text: label }
            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                QuietButton {
                    text: "Choose"
                    Layout.preferredWidth: 82
                    enabled: !appController.busy
                    onClicked: pathPickerRoot.pick()
                }
                Rectangle {
                    Layout.fillWidth: true
                    height: 32
                    color: panelAlt
                    border.color: line
                    radius: 4
                    Text {
                        anchors.fill: parent
                        anchors.leftMargin: 8
                        anchors.rightMargin: 8
                        text: path.length > 0 ? path : "No folder selected"
                        color: path.length > 0 ? muted : faint
                        font.family: root.mono
                        font.pixelSize: 11
                        verticalAlignment: Text.AlignVCenter
                        elide: Text.ElideMiddle
                    }
                }
            }
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
                                            text: {
                                                switch(model.state) {
                                                case 0: return "Pending"
                                                case 1: return "Scanning"
                                                case 2: return "Copying"
                                                case 3: return "Verifying"
                                                case 4: return "Complete"
                                                case 5: return "Failed"
                                                }
                                            }
                                            color: model.state === 4 ? green : (model.state === 5 ? bad : warn)
                                            font.family: root.mono
                                            font.pixelSize: 10
                                            horizontalAlignment: Text.AlignRight
                                            Layout.fillWidth: true
                                        }
                                        ProgressBar {
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
                                        danger: true
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
                    CheckBox {
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

                    QuietButton {
                        Layout.fillWidth: true
                        height: 38
                        text: appController.busy ? "Offloading..." : "Start Offload"
                        danger: true
                        enabled: !appController.busy && appController.destinationModel.count > 0
                        onClicked: appController.startOffload()
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        height: 38
                        text: "Cancel"
                        danger: true
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
                            Text {
                                text: metadataExpanded ? "Collapse ▼" : "Expand ▶"
                                color: faint
                                font.family: root.mono
                                font.pixelSize: 10
                            }
                            MouseArea {
                                anchors.fill: parent
                                onClicked: metadataExpanded = !metadataExpanded
                                cursorShape: Qt.PointingHandCursor
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
                                onEditingFinished: appController.projectName = text
                            }
                            FieldLabel { text: "Shoot date" }
                            DenseTextField {
                                Layout.fillWidth: true
                                text: appController.shootDate
                                placeholderText: "YYYY-MM-DD"
                                enabled: !appController.busy
                                onEditingFinished: appController.shootDate = text
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
                                        onEditingFinished: appController.cardName = text
                                    }
                                }
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    FieldLabel { text: "Camera ID" }
                                    DenseTextField {
                                        Layout.fillWidth: true
                                        text: appController.cameraId
                                        enabled: !appController.busy
                                        onEditingFinished: appController.cameraId = text
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
                        color: appController.pass ? (dark ? "#1a2a1f" : "#e0f0e6") : panelAlt
                        border.color: appController.pass ? green : line
                        radius: 4
                        visible: appController.canExport
                        Column {
                            anchors.fill: parent
                            anchors.margins: 9
                            spacing: 5
                            MetaLabel { text: "Status" }
                            Text {
                                text: appController.pass ? "PASS" : "FAIL"
                                color: appController.pass ? green : bad
                                font.family: root.mono
                                font.pixelSize: 18
                                font.bold: true
                            }
                        }
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
                    QuietButton { text: "MHL"; enabled: appController.canExport && !appController.busy; onClicked: appController.exportMhl() }
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
                        anchors.centerIn: parent
                        width: 480
                        height: 220
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
                        ProgressBar {
                            Layout.preferredWidth: 180
                            from: 0
                            to: 1
                            value: appController.overallProgress
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
                    }
                    Text {
                        text: appController.currentFile
                        color: faint
                        font.family: root.mono
                        font.pixelSize: 10
                        elide: Text.ElideMiddle
                        Layout.fillWidth: true
                        visible: appController.busy && appController.currentFile.length > 0
                    }
                    Rectangle { Layout.fillWidth: true; height: 1; color: line }
                    ListView {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        clip: true
                        model: appController.logLines
                        delegate: Text {
                            width: ListView.view.width
                            text: modelData
                            color: muted
                            font.family: root.mono
                            font.pixelSize: 10
                            elide: Text.ElideRight
                        }
                    }
                }
            }
        }
    }
}
