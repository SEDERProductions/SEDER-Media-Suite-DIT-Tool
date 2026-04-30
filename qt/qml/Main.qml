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
    readonly property var columnWidths: [118, 440, 110, 110, 280, 280]

    color: bg

    function colWidth(column) {
        return columnWidths[column] || 160
    }

    function toneColor(tone) {
        if (tone === "good") return green
        if (tone === "warn") return warn
        if (tone === "bad") return bad
        return faint
    }

    function statusColor(status) {
        if (status === 0) return green
        if (status === 1) return bad
        if (status === 2 || status === 3 || status === 4 || status === 5) return warn
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

    component MetricBox: Rectangle {
        property string label: ""
        property string value: ""
        property string tone: "neutral"
        Layout.fillWidth: true
        Layout.preferredHeight: 58
        color: panelAlt
        border.color: line
        radius: 4
        Column {
            anchors.fill: parent
            anchors.margins: 9
            spacing: 5
            MetaLabel { text: label }
            Text {
                text: value
                color: toneColor(tone)
                font.family: root.mono
                font.pixelSize: 18
                font.bold: true
                elide: Text.ElideRight
                width: parent.width
            }
        }
    }

    component FilterButton: Button {
        property int filterValue: 0
        property bool selected: filterModel.filter === filterValue
        height: 30
        font.family: root.sans
        font.pixelSize: 12
        onClicked: appController.setFilter(filterValue)
        contentItem: Text {
            text: parent.text
            color: parent.selected ? "#ffffff" : ink
            font: parent.font
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        background: Rectangle {
            color: parent.selected ? red : panelAlt
            border.color: parent.selected ? red : line
            radius: 4
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
            Layout.preferredWidth: 78
            color: dark ? "#16140f" : "#2a261d"

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 10
                spacing: 10
                Text {
                    text: "REC"
                    color: red
                    font.family: root.mono
                    font.pixelSize: 12
                    font.bold: true
                    Layout.alignment: Qt.AlignHCenter
                }
                Rectangle { Layout.fillWidth: true; height: 1; color: "#5c5143" }
                Rectangle {
                    Layout.fillWidth: true
                    height: 44
                    radius: 4
                    color: red
                    Text {
                        anchors.centerIn: parent
                        text: "DIT"
                        color: "#ffffff"
                        font.family: root.mono
                        font.pixelSize: 11
                        font.bold: true
                    }
                }
                Repeater {
                    model: ["CMP", "RW", "INS", "CAP"]
                    Rectangle {
                        Layout.fillWidth: true
                        height: 40
                        radius: 4
                        color: "transparent"
                        border.color: "#5c5143"
                        opacity: 0.42
                        Text {
                            anchors.centerIn: parent
                            text: modelData
                            color: "#ece6d9"
                            font.family: root.mono
                            font.pixelSize: 10
                        }
                    }
                }
                Item { Layout.fillHeight: true }
                Text {
                    text: "LOCAL"
                    color: "#ada596"
                    font.family: root.mono
                    font.pixelSize: 10
                    Layout.alignment: Qt.AlignHCenter
                }
            }
        }

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

                    MetaLabel { text: "01 / Folders" }
                    PathPicker {
                        label: "Source folder"
                        path: appController.sourcePath
                        onPick: appController.chooseSourceFolder()
                    }
                    PathPicker {
                        label: "Destination folder"
                        path: appController.destinationPath
                        onPick: appController.chooseDestinationFolder()
                    }

                    MetaLabel { text: "02 / Metadata" }
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

                    MetaLabel { text: "03 / Verification" }
                    FieldLabel { text: "Compare mode" }
                    ComboBox {
                        Layout.fillWidth: true
                        model: ["Path + Size", "Path + Size + Modified Time", "Path + Size + Checksum"]
                        currentIndex: appController.compareMode
                        enabled: !appController.busy
                        onActivated: appController.compareMode = index
                        font.family: root.sans
                        font.pixelSize: 12
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
                        Layout.preferredHeight: 76
                        text: appController.ignorePatterns
                        enabled: !appController.busy
                        color: ink
                        font.family: root.mono
                        font.pixelSize: 11
                        wrapMode: TextArea.Wrap
                        onTextChanged: appController.ignorePatterns = text
                        background: Rectangle { color: panelAlt; border.color: line; radius: 4 }
                    }

                    QuietButton {
                        Layout.fillWidth: true
                        height: 38
                        text: appController.busy ? "Verifying..." : "Start Verification"
                        danger: true
                        enabled: !appController.busy
                        onClicked: appController.startComparison()
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
                    MetricBox { label: "Matching"; value: appController.matchingCount.toString(); tone: "good" }
                    MetricBox { label: "Changed"; value: appController.changedCount.toString(); tone: appController.changedCount > 0 ? "bad" : "neutral" }
                    MetricBox { label: "Only A"; value: appController.onlyACount.toString(); tone: appController.onlyACount > 0 ? "warn" : "neutral" }
                    MetricBox { label: "Only B"; value: appController.onlyBCount.toString(); tone: appController.onlyBCount > 0 ? "warn" : "neutral" }
                    MetricBox { label: "Files"; value: appController.totalFiles.toString(); tone: "neutral" }
                    MetricBox { label: "Size"; value: appController.formatBytes(appController.totalSize); tone: "neutral" }
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
                    FilterButton { text: "All"; filterValue: 0 }
                    FilterButton { text: "Matching"; filterValue: 1 }
                    FilterButton { text: "Changed"; filterValue: 2 }
                    FilterButton { text: "Only A"; filterValue: 3 }
                    FilterButton { text: "Only B"; filterValue: 4 }
                    FilterButton { text: "Folders"; filterValue: 5 }
                    Item { Layout.fillWidth: true }
                    QuietButton { text: "TXT"; enabled: appController.canExport && !appController.busy; onClicked: appController.exportTxt() }
                    QuietButton { text: "CSV"; enabled: appController.canExport && !appController.busy; onClicked: appController.exportCsv() }
                    QuietButton { text: "MHL"; enabled: appController.mhlAvailable && !appController.busy; onClicked: appController.exportMhl() }
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

                    Row {
                        Layout.fillWidth: true
                        height: 30
                        Repeater {
                            model: ["Status", "Relative Path", "Size A", "Size B", "Checksum A", "Checksum B"]
                            Rectangle {
                                width: root.colWidth(index)
                                height: 30
                                color: panel
                                border.color: line
                                MetaLabel {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    verticalAlignment: Text.AlignVCenter
                                    text: modelData
                                }
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true

                        TableView {
                            id: table
                            anchors.fill: parent
                            clip: true
                            model: filterModel
                            boundsBehavior: Flickable.StopAtBounds
                            columnSpacing: 0
                            rowSpacing: 0
                            columnWidthProvider: function(column) { return root.colWidth(column) }
                            rowHeightProvider: function(row) { return 32 }
                            ScrollBar.vertical: ScrollBar {}
                            ScrollBar.horizontal: ScrollBar {}

                            delegate: Rectangle {
                                implicitWidth: table.columnWidthProvider(column)
                                implicitHeight: 32
                                color: row % 2 === 0 ? panel : panelAlt
                                border.color: line
                                Text {
                                    anchors.fill: parent
                                    anchors.leftMargin: 8
                                    anchors.rightMargin: 8
                                    verticalAlignment: Text.AlignVCenter
                                    text: display
                                    color: column === 0 ? statusColor(status) : ink
                                    font.family: column === 1 || column >= 4 ? root.mono : root.sans
                                    font.pixelSize: column >= 4 ? 10 : 12
                                    font.bold: column === 0
                                    elide: column === 1 ? Text.ElideMiddle : Text.ElideRight
                                }
                            }
                        }

                        Rectangle {
                            visible: filterModel.visibleRowCount === 0 && !appController.busy
                            anchors.centerIn: parent
                            width: 420
                            height: 110
                            color: "transparent"
                            Column {
                                anchors.centerIn: parent
                                spacing: 8
                                Text {
                                    text: appController.canExport ? "No Rows Match Filter" : "Ready For Verification"
                                    color: ink
                                    font.family: root.sans
                                    font.pixelSize: 20
                                    font.bold: true
                                    horizontalAlignment: Text.AlignHCenter
                                    width: 420
                                }
                                Text {
                                    text: appController.canExport ? "Change the filter to inspect the current report." : "Choose source and destination folders, then start verification."
                                    color: muted
                                    font.family: root.sans
                                    font.pixelSize: 13
                                    horizontalAlignment: Text.AlignHCenter
                                    width: 420
                                    wrapMode: Text.WordWrap
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
                            value: appController.progress
                            visible: appController.busy || appController.progress > 0
                        }
                        ComboBox {
                            Layout.preferredWidth: 112
                            model: ["system", "light", "dark"]
                            currentIndex: themeController.preference === "dark" ? 2 : (themeController.preference === "light" ? 1 : 0)
                            onActivated: themeController.preference = model[index]
                            font.family: root.sans
                            font.pixelSize: 11
                        }
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
