import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

Rectangle {
    id: header
    signal openSource()
    signal addDestination()
    signal openPreferences()

    implicitHeight: 54
    color: Theme.surface.panel

    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: Theme.space4
        anchors.rightMargin: Theme.space4
        spacing: Theme.space3

        Rectangle {
            Layout.preferredWidth: 30
            Layout.preferredHeight: 30
            radius: Theme.radiusSm
            color: Theme.accent.brand
            Icon { anchors.centerIn: parent; name: "layers"; size: 19; color: Theme.text.onAccent; stroke: 2 }
        }
        ColumnLayout {
            spacing: 0
            Text {
                text: "SEDER Media Suite"
                color: Theme.text.hi
                font.family: Theme.fontSans
                font.pixelSize: Theme.textLabel
                font.bold: true
            }
            Text {
                text: "DIT · Offload Verification"
                color: Theme.text.faint
                font.family: Theme.fontMono
                font.pixelSize: 9
                font.letterSpacing: 0.5
                font.capitalization: Font.AllUppercase
            }
        }

        Item { Layout.fillWidth: true }

        QuietButton {
            text: "Open Source"
            iconName: "folder"
            enabled: !appController.busy
            onClicked: header.openSource()
        }
        QuietButton {
            text: "Add Destination"
            iconName: "plus"
            enabled: !appController.busy
            onClicked: header.addDestination()
        }

        Divider { Layout.preferredHeight: 24; vertical: true }

        Icon { name: "drop"; size: 14; color: Theme.text.faint }
        StyledComboBox {
            Layout.preferredWidth: 100
            model: ["Auto", "Light", "Dark"]
            currentIndex: themeController.preference === "dark" ? 2
                : (themeController.preference === "light" ? 1 : 0)
            onActivated: {
                const map = ["system", "light", "dark"]
                themeController.preference = map[currentIndex]
            }
        }
        IconButton {
            iconName: "sliders"
            iconSize: 17
            Accessible.name: "Preferences"
            ToolTip.visible: hovered
            ToolTip.text: "Preferences"
            onClicked: header.openPreferences()
        }
    }

    Divider { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom }
}
