import QtQuick
import QtQuick.Layouts
import SederDit

// Small rounded status badge: optional icon + uppercase label, tinted by tone.
Rectangle {
    id: pill

    property string text: ""
    property string icon: ""
    // neutral | good | warn | bad | info
    property string tone: "neutral"

    readonly property color fg: tone === "good" ? Theme.accent.success
        : tone === "warn" ? Theme.accent.warning
        : tone === "bad" ? Theme.accent.danger
        : tone === "info" ? Theme.accent.brand
        : Theme.text.mid
    readonly property color bg: tone === "good" ? Theme.accent.successBg
        : tone === "warn" ? Theme.accent.warningBg
        : tone === "bad" ? Theme.accent.dangerBg
        : Theme.surface.raised

    implicitHeight: 20
    implicitWidth: row.implicitWidth + Theme.space3 * 2
    radius: height / 2
    color: bg

    RowLayout {
        id: row
        anchors.centerIn: parent
        spacing: 4
        Icon {
            visible: pill.icon !== ""
            name: pill.icon
            size: 12
            color: pill.fg
            Layout.alignment: Qt.AlignVCenter
        }
        Text {
            text: pill.text
            color: pill.fg
            font.family: Theme.fontMono
            font.pixelSize: Theme.textCaption
            font.bold: true
            font.letterSpacing: 0.4
            font.capitalization: Font.AllUppercase
            Layout.alignment: Qt.AlignVCenter
        }
    }
}
