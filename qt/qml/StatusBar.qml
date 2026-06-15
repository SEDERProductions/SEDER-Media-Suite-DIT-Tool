import QtQuick
import QtQuick.Layouts
import SederDit

Rectangle {
    id: bar
    implicitHeight: 42
    color: Theme.surface.panel

    Divider { anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top }

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.space4
        anchors.rightMargin: Theme.space4
        spacing: Theme.space3

        Icon {
            name: appController.busy ? "activity" : "check"
            size: 14
            color: appController.busy ? Theme.accent.warning : Theme.text.faint
        }
        Text {
            Layout.fillWidth: true
            text: appController.busy && appController.currentFile.length > 0
                ? appController.statusText + " — " + appController.currentFile
                : appController.statusText
            color: Theme.text.mid
            font.family: Theme.fontSans
            font.pixelSize: Theme.textBody
            elide: Text.ElideMiddle
        }

        StyledProgressBar {
            Layout.preferredWidth: 220
            from: 0
            to: 1
            value: appController.overallProgress
            indeterminate: appController.busy
                && appController.statusText === "Scanning source..."
                && appController.overallProgress <= 0
            visible: appController.busy || appController.overallProgress > 0
        }

        Text {
            text: appController.logLines.length + " log"
            color: Theme.text.faint
            font.family: Theme.fontMono
            font.pixelSize: Theme.textCaption
        }
        QuietButton {
            text: "Copy"
            iconName: "clipboard"
            enabled: appController.logLines.length > 0
            onClicked: appController.copyLog()
        }
        QuietButton {
            text: "Clear"
            iconName: "trash"
            enabled: appController.logLines.length > 0
            onClicked: appController.clearLog()
        }
    }
}
