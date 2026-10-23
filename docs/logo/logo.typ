// This board's logo; `just logo` renders it to logo.svg and logo-dark.svg.
#import "template_logo.typ": template-logo

#let values = (board: "Nucleo-G0B1RE", chip: "G0B1", subtitle: "WireWeaver firmware template")

#show: template-logo.with(..values, theme: sys.inputs.at("theme", default: "light"))
