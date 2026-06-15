import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SederDit

Dialog {
    id: aboutDialog
    modal: true
    title: "About SEDER Media Suite DIT"
    standardButtons: Dialog.Close

    background: Rectangle {
        color: Theme.surface.panel
        border.color: Theme.border.strong
        radius: Theme.radiusMd
    }

    ColumnLayout {
        spacing: Theme.space3
        width: 440

        RowLayout {
            spacing: Theme.space3
            Rectangle {
                Layout.preferredWidth: 44
                Layout.preferredHeight: 44
                radius: Theme.radiusMd
                color: Theme.accent.brand
                Icon { anchors.centerIn: parent; name: "layers"; size: 24; color: Theme.text.onAccent; stroke: 2 }
            }
            ColumnLayout {
                spacing: 2
                Text {
                    text: "SEDER Media Suite DIT"
                    color: Theme.text.hi
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.textTitle
                    font.bold: true
                }
                Text {
                    text: "Version " + (appController.appVersion || "")
                    color: Theme.text.mid
                    font.family: Theme.fontMono
                    font.pixelSize: Theme.textBody
                }
            }
        }

        Divider { Layout.fillWidth: true }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Local-first DIT offload verification for source and destination transfers. "
                + "A Qt 6 / QML interface over a Rust core for recursive scans, multi-algorithm "
                + "checksums, multi-destination copies, clip metadata, and TXT / CSV / MHL / ALE reports."
            color: Theme.text.mid
            font.family: Theme.fontSans
            font.pixelSize: Theme.textBody
        }

        Text {
            text: "© Seder Productions · Released under GPL-3.0-only."
            color: Theme.text.faint
            font.family: Theme.fontSans
            font.pixelSize: Theme.textMeta
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "<a href=\"https://github.com/sederproductions/seder-dit-tool\">github.com/sederproductions/seder-dit-tool</a>"
            color: Theme.accent.success
            linkColor: Theme.accent.success
            font.family: Theme.fontSans
            font.pixelSize: Theme.textBody
            onLinkActivated: (url) => Qt.openUrlExternally(url)
            MouseArea {
                anchors.fill: parent
                acceptedButtons: Qt.NoButton
                cursorShape: parent.hoveredLink !== "" ? Qt.PointingHandCursor : Qt.ArrowCursor
            }
        }
    }
}
