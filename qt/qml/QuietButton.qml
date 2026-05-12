import QtQuick
import QtQuick.Controls

Button {
    property string variant: "neutral"

    readonly property bool dark: themeController.dark
    readonly property color green: dark ? "#4cab7e" : "#1f7a4d"
    readonly property color red: dark ? "#d1411a" : "#c63b13"
    readonly property color panelAlt: dark ? "#282521" : "#e3dccb"
    readonly property color ink: dark ? "#ece6d9" : "#16140f"
    readonly property color faint: dark ? "#716a5f" : "#7a7363"
    readonly property color line: dark ? "#3a352e" : "#d6cfbe"
    readonly property string sans: "Manrope, Helvetica Neue, Helvetica, Arial, sans-serif"

    readonly property color variantBase: {
        if (variant === "primary") return green
        if (variant === "danger") return red
        return panelAlt
    }
    readonly property color variantHover: {
        if (variant === "primary") return dark ? "#5ab98d" : "#2f8c5d"
        if (variant === "danger") return dark ? "#e2532d" : "#d84b22"
        return dark ? "#332f2a" : "#ded6c5"
    }
    readonly property color variantFocus: {
        if (variant === "primary") return dark ? "#67c59a" : "#3f9a69"
        if (variant === "danger") return dark ? "#ef6240" : "#e05a33"
        return dark ? "#3a352e" : "#d6cfbe"
    }
    readonly property color variantBorder: {
        if (variant === "neutral") return line
        return variantBase
    }
    readonly property color variantText: variant === "neutral" ? ink : "#ffffff"

    height: 32
    font.family: sans
    font.pixelSize: 12
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus

    contentItem: Text {
        text: parent.text
        color: parent.enabled ? parent.variantText : (parent.variant === "neutral" ? faint : "#f4eee5")
        font: parent.font
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }
    background: Rectangle {
        color: {
            if (!parent.enabled)
                return parent.variant === "neutral" ? panelAlt : Qt.darker(parent.variantBase, 1.16)
            if (parent.down || parent.visualFocus)
                return parent.variantFocus
            if (parent.hovered)
                return parent.variantHover
            return parent.variantBase
        }
        border.color: parent.enabled ? (parent.visualFocus ? parent.variantFocus : parent.variantBorder)
                                   : (parent.variant === "neutral" ? line : Qt.darker(parent.variantBase, 1.25))
        border.width: parent.visualFocus ? 2 : 1
        radius: 4
        opacity: parent.enabled ? 1 : 0.6
    }
}
