import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Rectangle {
    id: pathPickerRoot
    property string label: ""
    property string path: ""
    property bool busy: false
    property var recents: []
    signal pick()
    signal acceptDroppedPath(string droppedPath)
    signal recentSelected(string recentPath)

    Layout.fillWidth: true
    height: 68
    color: "transparent"

    readonly property bool dark: themeController.dark
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property color highlight: dark ? "#4cab7e" : "#1f7a4d"
    readonly property string mono: "Menlo, Consolas, monospace"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    function urlToLocalPath(url) {
        const s = url.toString()
        if (s.startsWith("file:///")) {
            // Windows: file:///C:/foo  → C:/foo. Unix: file:///home/x → /home/x
            const stripped = s.substring(8)
            return stripped.length >= 3 && stripped.charAt(1) === ":"
                ? decodeURIComponent(stripped)
                : "/" + decodeURIComponent(stripped)
        }
        if (s.startsWith("file://")) {
            return decodeURIComponent(s.substring(7))
        }
        return decodeURIComponent(s)
    }

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
                enabled: !pathPickerRoot.busy
                onClicked: pathPickerRoot.pick()
            }
            Rectangle {
                id: pathField
                Layout.fillWidth: true
                height: 32
                color: dropArea.containsDrag ? Qt.darker(panelAlt, dark ? 0.9 : 1.05) : panelAlt
                border.color: dropArea.containsDrag ? highlight : line
                border.width: dropArea.containsDrag ? 2 : 1
                radius: 4
                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 8
                    anchors.rightMargin: 8
                    text: dropArea.containsDrag
                        ? "Drop folder to use"
                        : (path.length > 0 ? path : "No folder selected — drop a folder here or click Choose")
                    color: dropArea.containsDrag ? highlight : (path.length > 0 ? muted : faint)
                    font.family: mono
                    font.pixelSize: 11
                    verticalAlignment: Text.AlignVCenter
                    elide: Text.ElideMiddle
                }
                DropArea {
                    id: dropArea
                    anchors.fill: parent
                    enabled: !pathPickerRoot.busy
                    onDropped: (drop) => {
                        if (drop.hasUrls && drop.urls.length > 0) {
                            const local = pathPickerRoot.urlToLocalPath(drop.urls[0])
                            if (local && local.length > 0) {
                                pathPickerRoot.acceptDroppedPath(local)
                                drop.accept()
                            }
                        }
                    }
                }
            }
            QuietButton {
                id: recentsButton
                text: "Recent ▾"
                Layout.preferredWidth: 78
                enabled: !pathPickerRoot.busy && pathPickerRoot.recents && pathPickerRoot.recents.length > 0
                onClicked: recentsMenu.popup()
                Menu {
                    id: recentsMenu
                    Repeater {
                        model: pathPickerRoot.recents
                        MenuItem {
                            text: modelData
                            onTriggered: pathPickerRoot.recentSelected(modelData)
                        }
                    }
                }
            }
        }
    }
}
