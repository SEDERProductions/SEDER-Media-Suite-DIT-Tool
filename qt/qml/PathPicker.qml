import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

Rectangle {
    id: pathPickerRoot
    property string label: ""
    property string path: ""
    property bool busy: false
    property var recents: []
    property string placeholder: "No folder selected — drop a folder here or click Choose"
    signal pick()
    signal acceptDroppedPath(string droppedPath)
    signal recentSelected(string recentPath)

    Layout.fillWidth: true
    implicitHeight: column.implicitHeight
    color: "transparent"

    function urlToLocalPath(url) {
        const s = url.toString()
        if (s.startsWith("file:///")) {
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
        id: column
        width: parent.width
        spacing: 6
        FieldLabel { text: pathPickerRoot.label; visible: pathPickerRoot.label !== "" }
        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            QuietButton {
                text: "Choose"
                iconName: "folder"
                Layout.preferredWidth: 96
                enabled: !pathPickerRoot.busy
                onClicked: pathPickerRoot.pick()
            }
            Rectangle {
                id: pathField
                Layout.fillWidth: true
                height: Theme.fieldHeight
                color: dropArea.containsDrag ? Theme.accent.successBg : Theme.surface.raised
                border.color: dropArea.containsDrag ? Theme.accent.success : Theme.border.base
                border.width: dropArea.containsDrag ? Theme.focusRing : 1
                radius: Theme.radiusSm
                Behavior on border.color { ColorAnimation { duration: Theme.motionFast } }
                Text {
                    anchors.fill: parent
                    anchors.leftMargin: 10
                    anchors.rightMargin: 10
                    text: dropArea.containsDrag
                        ? "Drop folder to use"
                        : (pathPickerRoot.path.length > 0 ? pathPickerRoot.path : pathPickerRoot.placeholder)
                    color: dropArea.containsDrag ? Theme.accent.success
                        : (pathPickerRoot.path.length > 0 ? Theme.text.mid : Theme.text.faint)
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textMeta
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
                text: "Recent"
                iconName: "chevron-down"
                Layout.preferredWidth: 90
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
