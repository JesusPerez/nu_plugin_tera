# nu_plugin_tera

A [Nushell](https://nushell.sh/) plugin to use [Tera templates](https://keats.github.io/tera/docs/).

## Installing

Clone this repository 

> [!WARNING]  
> **nu_plugin_tera** has dependencies to nushell source via local path in Cargo.toml
> Nushell and plugins require to be **sync** with same **version** 

Clone [Nushell](https://nushell.sh/) to plugin to use [Tera templates](https://keats.github.io/tera/docs/) or change dependecies in [Cargo.toml](Cargo.toml)

This plugin is also included as submodule in [nushell-plugins](https://repo.jesusperez.pro/jesus/nushell-plugins) 
as part of plugins collection for [Provisioning project](https://rlung.librecloud.online/jesus/provisioning)

Build from source 

```nushell
> cd nu_plugin_tera
> cargo install --path .
```

### Nushell 

In a [Nushell](https://nushell.sh/)

```nushell
> plugin add ~/.cargo/bin/nu_plugin_tera
```

## Usage

```nushell
> tera-render <template> (context)
```

Flags:
- **-h**, **--help**: Display the help message for this command

Parameters:
- **template** <path>: Ruta al archivo .tera
- **context** <any>: Datos de contexto (record o JSON path) (optional)


### Examples

Render **template.tera** with a record as context from the pipeline.


**data.json**
```json
{
  "name": "Akasha",
  "projects": [
    {
      "name": "TheProject",
      "status": "active"
    }
  ]
}
```

**template.tera**
```jinja
Hello, {{ name }}!Projects:
{% for project in projects -%}
- {{ project.name }} ({{ project.status }})
{% endfor %}
```

### Other options

```nushell
> open data.json | wrap value | tera-render template.tera
> open data.json | tera-render template.tera
> { name: 'Akasha', projects: [ {'name': 'TheProject' , 'status': 'active' }]  } | tera-render template.tera
```

Result: 
<pre>
  Hello, Akasha!
  Projects:
  - TheProject (active)
</pre>

