{ name: "Akasha", project: "Nushell" } | render template.tera

render template.tera context.json

open data.json | render template.tera
