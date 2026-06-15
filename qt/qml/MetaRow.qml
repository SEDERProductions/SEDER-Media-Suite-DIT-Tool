import QtQuick
import QtQuick.Layouts
import SederDit

// Label / value row for the clip inspector.
RowLayout {
    property string label: ""
    property string value: ""
    property bool mono: true

    visible: value !== ""
    spacing: Theme.space2

    Text {
        text: label
        color: Theme.text.faint
        font.family: Theme.fontSans
        font.pixelSize: Theme.textMeta
        Layout.preferredWidth: 96
        Layout.alignment: Qt.AlignTop
    }
    Text {
        Layout.fillWidth: true
        text: value
        color: Theme.text.hi
        font.family: mono ? Theme.fontMono : Theme.fontSans
        font.pixelSize: Theme.textMeta
        wrapMode: Text.WrapAnywhere
    }
}
