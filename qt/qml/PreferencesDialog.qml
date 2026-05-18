import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Dialog {
    id: prefsDialog
    modal: true
    title: "Preferences"
    standardButtons: Dialog.Close

    readonly property bool dark: themeController.dark
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color muted: dark ? "#ada596" : "#4a4438"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color panel: dark ? "#1f1d1a" : "#f8f4ea"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"
    readonly property string mono: "Menlo, Consolas, monospace"

    background: Rectangle {
        color: panel
        border.color: line
        radius: 4
    }

    ColumnLayout {
        spacing: 14
        width: 480

        Text {
            text: "Appearance"
            color: ink
            font.family: sans
            font.pixelSize: 13
            font.bold: true
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            FieldLabel { text: "Theme" }
            ComboBox {
                Layout.fillWidth: true
                model: ["system", "light", "dark"]
                currentIndex: Math.max(0, model.indexOf(themeController.preference))
                onActivated: themeController.preference = model[currentIndex]
            }
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: line }

        Text {
            text: "Default offload options"
            color: ink
            font.family: sans
            font.pixelSize: 13
            font.bold: true
        }

        Text {
            text: "These values are used the next time the app starts."
            color: faint
            font.family: sans
            font.pixelSize: 11
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        StyledCheckBox {
            text: "Ignore hidden / system files by default"
            checked: settingsStore.defaultIgnoreHiddenSystem
            onToggled: settingsStore.defaultIgnoreHiddenSystem = checked
        }
        StyledCheckBox {
            text: "Verify after copy by default"
            checked: settingsStore.defaultVerifyAfterCopy
            onToggled: settingsStore.defaultVerifyAfterCopy = checked
        }
        StyledCheckBox {
            text: "Skip files already present at destination"
            checked: settingsStore.defaultSkipExisting
            onToggled: settingsStore.defaultSkipExisting = checked
        }
        StyledCheckBox {
            text: "Generate report by default"
            checked: settingsStore.defaultGenerateReport
            onToggled: settingsStore.defaultGenerateReport = checked
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 4
            FieldLabel { text: "Default ignore patterns (comma-separated)" }
            TextField {
                Layout.fillWidth: true
                text: settingsStore.defaultIgnorePatterns
                font.family: mono
                font.pixelSize: 11
                onEditingFinished: settingsStore.defaultIgnorePatterns = text
            }
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            FieldLabel { text: "Default checksum algorithm" }
            ComboBox {
                id: algoCombo
                readonly property var algos: ["BLAKE3", "MD5", "SHA1", "XXH3-64", "XXH3-128"]
                Layout.fillWidth: true
                model: algos
                currentIndex: Math.max(0, algos.indexOf(settingsStore.defaultChecksumAlgorithm))
                onActivated: settingsStore.defaultChecksumAlgorithm = algos[currentIndex]
            }
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "BLAKE3 (default) is fast and cryptographic. XXH3-64 is fastest and good for "
                + "in-house verification. MD5 / SHA-1 are slower but interoperate with legacy DIT pipelines."
            color: faint
            font.family: sans
            font.pixelSize: 11
        }

        Rectangle { Layout.fillWidth: true; height: 1; color: line }

        Text {
            text: "Destination subfolder template"
            color: ink
            font.family: sans
            font.pixelSize: 13
            font.bold: true
        }
        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: "Optional. When set, picking a destination folder will create and use the "
                + "expanded subfolder underneath it. Tokens: {project}, {date}, {card}, {camera}."
            color: faint
            font.family: sans
            font.pixelSize: 11
        }
        TextField {
            id: templateField
            Layout.fillWidth: true
            text: settingsStore.destinationTemplate
            placeholderText: "{project}/{date}/{card}"
            font.family: mono
            font.pixelSize: 11
            onEditingFinished: settingsStore.destinationTemplate = text
        }
        Text {
            id: templatePreview
            Layout.fillWidth: true
            wrapMode: Text.WrapAnywhere
            text: "Preview: " + (settingsStore.destinationTemplate.length > 0
                ? appController.previewDestinationTemplate("")
                : "(no template — destination folder used as-is)")
            color: muted
            font.family: mono
            font.pixelSize: 11
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 8
            Item { Layout.fillWidth: true }
            QuietButton {
                text: "Reset to factory defaults"
                onClicked: settingsStore.resetDefaultsToFactory()
            }
            QuietButton {
                text: "Apply to current session"
                onClicked: {
                    appController.applyDefaultsFromSettings()
                    prefsDialog.close()
                }
            }
        }
    }
}
