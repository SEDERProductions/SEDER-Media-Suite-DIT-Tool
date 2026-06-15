import QtQuick
import QtQuick.Layouts
import SederDit

Rectangle {
    id: rail
    property int currentMode: 0
    signal modeRequested(int mode)

    readonly property var items: [
        { icon: "hard-drive", label: "Offload" },
        { icon: "grid",       label: "Library" },
        { icon: "file-text",  label: "Reports" },
        { icon: "list",       label: "Queue" }
    ]

    implicitWidth: 78
    color: Theme.surface.backdrop

    Accessible.role: Accessible.PageTabList

    ColumnLayout {
        anchors.fill: parent
        anchors.topMargin: Theme.space3
        anchors.bottomMargin: Theme.space3
        spacing: Theme.space1

        Repeater {
            model: rail.items
            delegate: Item {
                id: cell
                required property int index
                required property var modelData
                readonly property bool active: rail.currentMode === index

                Layout.fillWidth: true
                Layout.preferredHeight: 62

                Accessible.role: Accessible.PageTab
                Accessible.name: modelData.label
                Accessible.focusable: true

                // Active indicator bar
                Rectangle {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 3
                    height: 30
                    radius: 1.5
                    color: Theme.accent.brand
                    visible: cell.active
                }

                Rectangle {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space2
                    anchors.rightMargin: Theme.space2
                    radius: Theme.radiusMd
                    color: cell.active ? Theme.surface.raised
                        : (hover.containsMouse ? Theme.surface.panel : "transparent")
                    Behavior on color { ColorAnimation { duration: Theme.motionFast } }

                    ColumnLayout {
                        anchors.centerIn: parent
                        spacing: 4
                        Icon {
                            Layout.alignment: Qt.AlignHCenter
                            name: cell.modelData.icon
                            size: 22
                            stroke: 1.9
                            color: cell.active ? Theme.accent.brand : Theme.text.mid
                        }
                        Text {
                            Layout.alignment: Qt.AlignHCenter
                            text: cell.modelData.label
                            color: cell.active ? Theme.text.hi : Theme.text.mid
                            font.family: Theme.fontSans
                            font.pixelSize: Theme.textCaption
                            font.bold: cell.active
                        }
                    }

                    MouseArea {
                        id: hover
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: rail.modeRequested(cell.index)
                    }
                }
            }
        }

        Item { Layout.fillHeight: true }
    }

    Divider { anchors.right: parent.right; anchors.top: parent.top; anchors.bottom: parent.bottom; vertical: true }
}
