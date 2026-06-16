import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit
import "Format.js" as Fmt

// Offload workspace: resizable config sidebar (left) + results pane (right).
// Ports the original Main.qml offload UI; all appController bindings preserved.
SplitView {
    id: offload
    orientation: Qt.Horizontal

    property bool metadataExpanded: false
    property bool logAutoScrollEnabled: true

    handle: Rectangle {
        implicitWidth: 6
        color: SplitHandle.pressed ? Theme.accent.brand
            : (SplitHandle.hovered ? Theme.surface.hover : Theme.surface.backdrop)
        Behavior on color { ColorAnimation { duration: Theme.motionFast } }
        Rectangle {
            anchors.centerIn: parent
            width: 1; height: parent.height
            color: Theme.border.base
        }
    }

    // ============================== CONFIG SIDEBAR ==============================
    Rectangle {
        SplitView.preferredWidth: 392
        SplitView.minimumWidth: 320
        SplitView.maximumWidth: 560
        color: Theme.surface.base

        ScrollView {
            anchors.fill: parent
            contentWidth: availableWidth
            clip: true
            ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

            ColumnLayout {
                width: offload.width > 0 ? parent.width : 0
                spacing: Theme.space3

                // ---- 01 / Source ----
                Panel {
                    Layout.fillWidth: true
                    Layout.topMargin: Theme.space4
                    Layout.leftMargin: Theme.space4
                    Layout.rightMargin: Theme.space4
                    title: "01 · Source"
                    icon: "folder"
                    PathPicker {
                        Layout.fillWidth: true
                        label: "Source folder"
                        path: appController.sourcePath
                        busy: appController.busy
                        recents: settingsStore ? settingsStore.recentSources : []
                        onPick: appController.chooseSourceFolder()
                        onAcceptDroppedPath: (droppedPath) => appController.addSourceFromPath(droppedPath)
                        onRecentSelected: (recentPath) => appController.addSourceFromPath(recentPath)
                    }
                }

                // ---- 02 / Destinations ----
                Panel {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.space4
                    Layout.rightMargin: Theme.space4
                    title: "02 · Destinations"
                    icon: "hard-drive"
                    bodySpacing: Theme.space2

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: destDropArea.containsDrag ? 46 : 0
                        color: Theme.accent.successBg
                        border.color: Theme.accent.success
                        border.width: 2
                        radius: Theme.radiusSm
                        visible: destDropArea.containsDrag
                        clip: true
                        Text {
                            anchors.centerIn: parent
                            text: "Drop folder to add as destination"
                            color: Theme.accent.success
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.textBody
                        }
                    }

                    DropArea {
                        id: destDropArea
                        Layout.fillWidth: true
                        Layout.preferredHeight: appController.destinationModel.count === 0 ? 64 : 0
                        enabled: !appController.busy
                        onDropped: (drop) => {
                            if (drop.hasUrls) {
                                for (let i = 0; i < drop.urls.length; ++i) {
                                    const s = drop.urls[i].toString()
                                    let local = s
                                    if (s.startsWith("file:///")) {
                                        const stripped = s.substring(8)
                                        local = (stripped.length >= 3 && stripped.charAt(1) === ":")
                                            ? decodeURIComponent(stripped)
                                            : "/" + decodeURIComponent(stripped)
                                    } else if (s.startsWith("file://")) {
                                        local = decodeURIComponent(s.substring(7))
                                    } else {
                                        local = decodeURIComponent(s)
                                    }
                                    if (local && local.length > 0)
                                        appController.addDestinationFromPath(local)
                                }
                                drop.accept()
                            }
                        }
                        Rectangle {
                            anchors.fill: parent
                            visible: appController.destinationModel.count === 0
                            radius: Theme.radiusSm
                            color: "transparent"
                            border.color: Theme.border.subtle
                            border.width: 1
                            Text {
                                anchors.centerIn: parent
                                width: parent.width - Theme.space4
                                horizontalAlignment: Text.AlignHCenter
                                text: "Drag folders here, or click Add Destination"
                                color: Theme.text.faint
                                font.family: Theme.fontSans
                                font.pixelSize: Theme.textMeta
                                wrapMode: Text.WordWrap
                            }
                        }
                    }

                    Repeater {
                        model: appController.destinationModel
                        delegate: Rectangle {
                            required property int index
                            required property var model
                            Layout.fillWidth: true
                            implicitHeight: 64
                            color: Theme.surface.raised
                            border.color: Theme.border.base
                            radius: Theme.radiusSm

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: Theme.space2
                                spacing: Theme.space2

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 2
                                    Text {
                                        text: model.label || "Destination"
                                        color: Theme.text.hi
                                        font.family: Theme.fontSans
                                        font.pixelSize: Theme.textBody
                                        font.bold: true
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    Text {
                                        text: model.path
                                        color: Theme.text.mid
                                        font.family: Theme.fontMono
                                        font.pixelSize: Theme.textCaption
                                        elide: Text.ElideMiddle
                                        Layout.fillWidth: true
                                    }
                                    StatusPill {
                                        text: Fmt.destStateLabel(model.state)
                                        icon: Fmt.destStateIcon(model.state)
                                        tone: Fmt.destStateTone(model.state)
                                        Accessible.name: "Destination status: " + Fmt.destStateLabel(model.state)
                                    }
                                    Text {
                                        visible: model.error && model.error.length > 0
                                        text: model.error
                                        color: Theme.accent.danger
                                        font.family: Theme.fontSans
                                        font.pixelSize: Theme.textCaption
                                        elide: Text.ElideRight
                                        Layout.fillWidth: true
                                    }
                                    StyledProgressBar {
                                        Layout.fillWidth: true
                                        from: 0; to: 1
                                        value: model.progress
                                        visible: model.state === 2 || model.state === 3
                                    }
                                }

                                ColumnLayout {
                                    Layout.alignment: Qt.AlignTop
                                    spacing: 2
                                    IconButton {
                                        iconName: "duplicate"
                                        enabled: !appController.busy
                                        Accessible.name: "Copy path to new destination"
                                        ToolTip.visible: hovered && enabled
                                        ToolTip.text: "Copy path to new destination"
                                        onClicked: appController.copyDestinationPath(index)
                                    }
                                    IconButton {
                                        iconName: "x"
                                        variant: "danger"
                                        enabled: !appController.busy
                                        Accessible.name: "Remove destination"
                                        onClicked: appController.removeDestination(index)
                                    }
                                }
                            }
                        }
                    }

                    QuietButton {
                        Layout.fillWidth: true
                        text: "Add Destination"
                        iconName: "plus"
                        enabled: !appController.busy
                        onClicked: appController.addDestinationFolder()
                    }
                }

                // ---- 03 / Options ----
                Panel {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.space4
                    Layout.rightMargin: Theme.space4
                    title: "03 · Options"
                    icon: "sliders"

                    SegmentedControl {
                        Layout.fillWidth: true
                        options: ["Verify after copy", "Copy only"]
                        currentIndex: appController.verifyAfterCopy ? 0 : 1
                        enabled: !appController.busy
                        onActivated: (index) => appController.verifyAfterCopy = (index === 0)
                    }
                    Text {
                        Layout.fillWidth: true
                        text: "MHL export requires Verify after copy."
                        color: Theme.text.mid
                        font.family: Theme.fontSans
                        font.pixelSize: Theme.textMeta
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
                    }
                    FieldLabel { text: "Ignore patterns" }
                    TextArea {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 62
                        text: appController.ignorePatterns
                        enabled: !appController.busy
                        color: Theme.text.hi
                        placeholderTextColor: Theme.text.faint
                        font.family: Theme.fontMono
                        font.pixelSize: Theme.textMeta
                        wrapMode: TextArea.Wrap
                        onTextChanged: appController.ignorePatterns = text
                        background: Rectangle {
                            color: Theme.surface.raised
                            border.color: Theme.border.base
                            radius: Theme.radiusSm
                        }
                    }
                    Divider { Layout.fillWidth: true }
                    QuietButton {
                        Layout.fillWidth: true
                        text: "Sync Destinations to Source"
                        iconName: "refresh"
                        enabled: !appController.busy && appController.destinationModel.count > 0 && appController.sourcePath.length > 0
                        onClicked: appController.syncDestinationPaths()
                        ToolTip.visible: hovered
                        ToolTip.text: "Replace last path component of all destinations with source folder name"
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        height: Theme.actionHeight
                        text: appController.busy ? "Offloading…" : "Start Offload"
                        iconName: appController.busy ? "activity" : "play"
                        variant: "primary"
                        enabled: !appController.busy && appController.destinationModel.count > 0
                        onClicked: appController.startOffload()
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        text: "Add to Queue"
                        iconName: "list"
                        enabled: appController.destinationModel.count > 0 && appController.sourcePath.length > 0
                        onClicked: appController.enqueueCurrent()
                        ToolTip.visible: hovered
                        ToolTip.text: "Stage this configuration as a job in the queue"
                    }
                    QuietButton {
                        Layout.fillWidth: true
                        height: Theme.actionHeight
                        text: "Cancel"
                        iconName: "stop"
                        variant: "danger"
                        visible: appController.busy
                        onClicked: appController.cancelOffload()
                    }
                }

                // ---- 04 / Metadata ----
                Panel {
                    Layout.fillWidth: true
                    Layout.leftMargin: Theme.space4
                    Layout.rightMargin: Theme.space4
                    Layout.bottomMargin: Theme.space4
                    title: "04 · Metadata"
                    icon: "calendar"
                    collapsible: true
                    collapsed: !offload.metadataExpanded
                    onCollapsedChanged: offload.metadataExpanded = !collapsed

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
                        spacing: Theme.space2
                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4
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
                            spacing: 4
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

    // ================================ RESULTS PANE ================================
    Rectangle {
        SplitView.fillWidth: true
        SplitView.minimumWidth: 420
        color: Theme.surface.base

        ColumnLayout {
            anchors.fill: parent
            anchors.margins: Theme.space4
            spacing: Theme.space3

            // ---- Stat cards ----
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space3

                StatCard {
                    label: "Files"
                    value: appController.totalFiles
                    iconName: "file-text"
                }
                StatCard {
                    label: "Size"
                    value: appController.formatBytes(appController.totalSize)
                    iconName: "hard-drive"
                }
                StatCard {
                    visible: appController.canExport
                    label: "Status"
                    value: appController.finalStatus === "PASS" ? "PASS"
                        : (appController.finalStatus === "COPIED (UNVERIFIED)" ? "UNVERIFIED" : "FAIL")
                    iconName: appController.finalStatus === "PASS" ? "check-circle"
                        : (appController.finalStatus === "COPIED (UNVERIFIED)" ? "alert" : "x-circle")
                    valueColor: appController.finalStatus === "PASS" ? Theme.accent.success
                        : (appController.finalStatus === "COPIED (UNVERIFIED)" ? Theme.accent.warning : Theme.accent.danger)
                    fillColor: appController.finalStatus === "PASS" ? Theme.accent.successBg
                        : (appController.finalStatus === "COPIED (UNVERIFIED)" ? Theme.accent.warningBg : Theme.accent.dangerBg)
                    accentColor: valueColor
                    subtitle: appController.finalStatus === "PASS" ? "All destination verifications passed"
                        : (appController.finalStatus === "COPIED (UNVERIFIED)" ? "Files copied but not verified"
                        : "One or more verifications failed")
                    Accessible.name: appController.finalStatus === "PASS"
                        ? "Pass status. All destination verifications passed."
                        : (appController.finalStatus === "COPIED (UNVERIFIED)"
                            ? "Copied unverified status. Files copied but not verified."
                            : "Fail status. One or more destination verifications failed.")
                }
            }

            // ---- Activity log header + export actions ----
            RowLayout {
                Layout.fillWidth: true
                spacing: Theme.space2
                Icon { name: "list"; size: 14; color: Theme.text.mid }
                MetaLabel { text: "Activity Log" }
                Text {
                    visible: appController.logLines.length > 0
                    text: "· " + appController.logLines.length + " entries"
                    color: Theme.text.faint
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textCaption
                }
                Item { Layout.fillWidth: true }
                QuietButton {
                    text: "TXT"; iconName: "file-text"
                    enabled: appController.canExport && !appController.busy
                    onClicked: appController.exportTxt()
                }
                QuietButton {
                    text: "CSV"; iconName: "file-text"
                    enabled: appController.canExport && !appController.busy
                    onClicked: appController.exportCsv()
                }
                QuietButton {
                    text: "MHL"; iconName: "file-text"
                    enabled: appController.canExportMhl && !appController.busy
                    visible: appController.canExportMhl || appController.canExport
                    onClicked: appController.exportMhl()
                    ToolTip.visible: hovered && !enabled
                    ToolTip.text: "Requires Verify after copy"
                }
            }

            // ---- Log card ----
            Panel {
                Layout.fillWidth: true
                Layout.fillHeight: true
                bodyPadding: Theme.space2

                EmptyState {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: 280
                    visible: appController.logLines.length === 0 && !appController.busy && !appController.canExport
                    iconName: "play"
                    title: "Ready for Offload"
                    subtitle: "1.  Choose a Source folder\n2.  Add one or more Destinations\n3.  Configure Options\n4.  Click Start Offload"
                }

                ListView {
                    id: logListView
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.preferredHeight: 280
                    visible: appController.logLines.length > 0 || appController.busy
                    clip: true
                    model: appController.logLines
                    spacing: 2
                    onCountChanged: {
                        if (offload.logAutoScrollEnabled && count > 0)
                            positionViewAtEnd()
                    }
                    onContentYChanged: {
                        const atBottom = (contentY + height) >= (contentHeight - 8)
                        offload.logAutoScrollEnabled = !appController.busy ? true : atBottom
                    }
                    delegate: RowLayout {
                        required property var modelData
                        width: ListView.view ? ListView.view.width : 0
                        spacing: Theme.space2
                        readonly property string severity: Fmt.logSeverity(modelData)
                        readonly property color sevColor: severity === "ERROR" ? Theme.accent.danger
                            : (severity === "WARN" ? Theme.accent.warning : Theme.text.mid)
                        Icon {
                            name: Fmt.severityIcon(severity)
                            size: 12
                            color: parent.sevColor
                            Layout.alignment: Qt.AlignVCenter
                        }
                        Text {
                            text: Fmt.logTimestamp(modelData)
                            color: Theme.text.faint
                            font.family: Theme.fontMono
                            font.pixelSize: Theme.textMeta
                        }
                        Text {
                            Layout.fillWidth: true
                            text: Fmt.logMessage(modelData)
                            color: parent.sevColor
                            font.family: Theme.fontMono
                            font.pixelSize: Theme.textMeta
                            elide: Text.ElideRight
                        }
                    }
                    ScrollBar.vertical: ScrollBar {}
                }
            }
        }
    }

    // Reusable stat card.
    component StatCard: Rectangle {
        property string label: ""
        property string value: ""
        property string iconName: ""
        property string subtitle: ""
        property color valueColor: Theme.text.hi
        property color fillColor: Theme.surface.raised
        property color accentColor: Theme.border.base

        Layout.fillWidth: true
        Layout.preferredHeight: 76
        radius: Theme.radiusMd
        color: fillColor
        border.color: accentColor

        RowLayout {
            anchors.fill: parent
            anchors.margins: Theme.space3
            spacing: Theme.space3
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 2
                MetaLabel { text: label }
                Text {
                    text: value
                    color: valueColor
                    font.family: Theme.fontMono
                    font.pixelSize: 20
                    font.bold: true
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                }
                Text {
                    visible: subtitle !== ""
                    text: subtitle
                    color: Theme.text.mid
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textCaption
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                }
            }
            Icon {
                name: iconName
                size: 22
                color: Qt.rgba(valueColor.r, valueColor.g, valueColor.b, 0.55)
                Layout.alignment: Qt.AlignTop
            }
        }
    }
}
