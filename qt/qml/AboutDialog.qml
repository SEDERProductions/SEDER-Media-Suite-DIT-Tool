import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: aboutDialog
    modal: true
    title: "About SEDER Media Suite DIT"
    standardButtons: Dialog.Close

    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color link: dark ? "#4cab7e" : "#1f7a4d"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string mono: "Menlo, Consolas, monospace"

    background: Rectangle {
        color: panel
        border.color: dark ? "#3a352e" : "#d6cfbe"
        radius: 4
    }

    ColumnLayout {
        spacing: 12
        width: 420

        Text {
            text: "SEDER Media Suite DIT"
            color: ink
            font.family: sans
            font.pixelSize: 18
            font.bold: true
        }

        Text {
            text: "Version " + (appController.appVersion || "")
            color: muted
            font.family: mono
            font.pixelSize: 12
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Local-first DIT folder verification for source and destination offloads. "
                + "Qt 6/QML interface with a Rust core for recursive scans, BLAKE3 checksums, "
                + "and TXT, CSV, and ASC MHL report exports."
            color: muted
            font.family: sans
            font.pixelSize: 12
        }

        Text {
            text: "© Seder Productions"
            color: faint
            font.family: sans
            font.pixelSize: 11
        }

        Text {
            text: "Released under GPL-3.0-only."
            color: faint
            font.family: sans
            font.pixelSize: 11
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "<a href=\"https://github.com/sederproductions/seder-dit-tool\">github.com/sederproductions/seder-dit-tool</a>"
            color: link
            linkColor: link
            font.family: sans
            font.pixelSize: 12
            onLinkActivated: (url) => Qt.openUrlExternally(url)
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.NoButton
                cursorShape: parent.hoveredLink !== "" ? Qt.PointingHandCursor : Qt.ArrowCursor
            }
        }
    }
}
