import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

// Grid of source media clips with lazy thumbnails. Never shows a broken
// image: a row falls back to a tinted format badge while its thumbnail is
// generating, when generation fails, or for non-visual kinds. Backed by
// MediaListModel (exposed as appController.mediaModel).
GridView {
    id: grid

    readonly property bool dark: themeController.dark
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string mono: "Menlo, Consolas, monospace"

    // Distinct tint per camera/media format so the badges read at a glance.
    function kindColor(kind) {
        switch (kind) {
        case "R3D": return red
        case "ARRI": return dark ? "#c99746" : "#9a6a16"
        case "BRAW": return dark ? "#4cab7e" : "#1f7a4d"
        case "Canon RAW": return dark ? "#d25645" : "#b43a1f"
        case "CinemaDNG": return dark ? "#7fa7d9" : "#3a6ea5"
        case "MXF": return dark ? "#b08cd9" : "#6a4a9a"
        case "MOV":
        case "MP4":
        case "MPEG-TS": return dark ? "#7fa7d9" : "#3a6ea5"
        case "Audio": return dark ? "#4cab7e" : "#1f7a4d"
        default: return faint
        }
    }

    clip: true
    cellWidth: 196
    cellHeight: 170
    boundsBehavior: Flickable.StopAtBounds
    ScrollBar.vertical: ScrollBar {}

    delegate: Item {
        width: grid.cellWidth
        height: grid.cellHeight

        Rectangle {
            anchors.fill: parent
            anchors.margins: 6
            color: grid.panel
            border.color: grid.line
            radius: 4

            Accessible.role: Accessible.Graphic
            Accessible.name: model.relativePath + ", " + model.kind + ", "
                             + appController.formatBytes(model.size)

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: 6
                spacing: 4

                // 16:9 thumbnail / placeholder.
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: width * 9 / 16
                    color: grid.panelAlt
                    border.color: grid.line
                    radius: 3
                    clip: true

                    Image {
                        anchors.fill: parent
                        source: model.thumbnailState === 2 ? model.thumbnailUrl : ""
                        visible: model.thumbnailState === 2
                        asynchronous: true
                        cache: false
                        fillMode: Image.PreserveAspectCrop
                    }

                    // Format badge while pending, on failure, or for
                    // non-visual kinds. A spinner shows only while generating.
                    Column {
                        anchors.centerIn: parent
                        visible: model.thumbnailState !== 2
                        spacing: 4
                        Text {
                            anchors.horizontalCenter: parent.horizontalCenter
                            text: model.kind
                            color: grid.kindColor(model.kind)
                            font.family: grid.sans
                            font.pixelSize: 16
                            font.bold: true
                        }
                        BusyIndicator {
                            anchors.horizontalCenter: parent.horizontalCenter
                            implicitWidth: 22
                            implicitHeight: 22
                            running: model.thumbnailState === 0 || model.thumbnailState === 1
                            visible: running
                        }
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: model.fileName
                    color: grid.ink
                    font.family: grid.sans
                    font.pixelSize: 11
                    elide: Text.ElideMiddle
                }
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        text: model.kind
                        color: grid.kindColor(model.kind)
                        font.family: grid.mono
                        font.pixelSize: 9
                        font.bold: true
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: appController.formatBytes(model.size)
                        color: grid.muted
                        font.family: grid.mono
                        font.pixelSize: 9
                    }
                }
            }
        }
    }
}
