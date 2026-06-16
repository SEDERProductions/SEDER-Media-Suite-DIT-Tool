import QtQuick
import QtQuick.Layouts
import SederDit

// Centered icon + title + subtitle, with an optional slot for extra controls.
Item {
    id: empty
    property string iconName: ""
    property string title: ""
    property string subtitle: ""
    default property alias extra: extraColumn.data

    ColumnLayout {
        anchors.centerIn: parent
        width: Math.min(empty.width - Theme.space6 * 2, 480)
        spacing: Theme.space3

        Rectangle {
            Layout.alignment: Qt.AlignHCenter
            width: 72; height: 72
            radius: Theme.radiusLg
            color: Theme.surface.raised
            border.color: Theme.border.subtle
            Icon {
                anchors.centerIn: parent
                name: empty.iconName
                size: 36
                stroke: 1.6
                color: Theme.text.faint
            }
        }
        Text {
            Layout.fillWidth: true
            text: empty.title
            horizontalAlignment: Text.AlignHCenter
            color: Theme.text.hi
            font.family: Theme.fontSans
            font.pixelSize: Theme.textTitle
            font.bold: true
        }
        Text {
            Layout.fillWidth: true
            visible: empty.subtitle !== ""
            text: empty.subtitle
            horizontalAlignment: Text.AlignHCenter
            wrapMode: Text.WordWrap
            color: Theme.text.mid
            font.family: Theme.fontSans
            font.pixelSize: Theme.textLabel
        }
        ColumnLayout {
            id: extraColumn
            Layout.fillWidth: true
            spacing: Theme.space2
        }
    }
}
