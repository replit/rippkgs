use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    // pub name: String,
    #[serde(default)]
    pub outputs: PackageOutputs,
    pub pname: Option<String>,
    pub version: Option<String>,
    pub meta: Option<PackageMeta>,
}

pub type PackageOutputs = std::collections::HashMap<String, PathBuf>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageMeta {
    pub available: Option<bool>,
    #[serde(default)]
    pub broken: bool,
    pub description: Option<String>,
    pub homepage: Option<OneOrList<String>>,
    #[serde(default)]
    pub insecure: bool,
    pub license: Option<serde_json::Value>,
    pub long_description: Option<String>,
    #[serde(default)]
    pub unfree: bool,
    #[serde(default)]
    pub unsupported: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum OneOrList<T> {
    One(T),
    List(Vec<T>),
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    #[test]
    fn one_or_list() {
        assert_matches!(
            serde_json::from_str::<super::OneOrList<String>>("[]"),
            Ok(super::OneOrList::List(_))
        );

        assert_matches!(
            serde_json::from_str::<super::OneOrList<String>>(r#""hi""#),
            Ok(super::OneOrList::One(_))
        );
    }
}

/*
{
  "name": "zsh-5.9",
  "outputName": "out",
  "outputs": {
    "doc": "/nix/store/a4mywc6011yz49bdri7ly1mkl1n0rsd6-zsh-5.9-doc",
    "info": "/nix/store/h35wq3a5a7k54ic0fkl4ml51w4w3mdld-zsh-5.9-info",
    "man": "/nix/store/dka6gvckn5l62nvbyhyvlz8pd504dsl3-zsh-5.9-man",
    "out": "/nix/store/l2gn5y5q6n51f36j47ysapmci0q7kdqb-zsh-5.9"
  },
  "pname": "zsh",
  "system": "aarch64-darwin",
  "version": "5.9",
  "meta": {
    "available": true,
    "broken": false,
    "description": "The Z shell",
    "homepage": "https://www.zsh.org/",
    "insecure": false,
    "license": "MIT-like",
    "longDescription": "Zsh is a UNIX command interpreter (shell) usable as an interactive login\nshell and as a shell script command processor.  Of the standard shells,\nzsh most closely resembles ksh but includes many enhancements.  Zsh has\ncommand line editing, builtin spelling correction, programmable command\ncompletion, shell functions (with autoloading), a history mechanism, and\na host of other features.\n",
    "maintainers": [
      {
        "email": "mail@pascal-wittmann.de",
        "github": "pSub",
        "githubId": 83842,
        "name": "Pascal Wittmann"
      },
      {
        "email": "artturin@artturin.com",
        "github": "Artturin",
        "githubId": 56650223,
        "matrix": "@artturin:matrix.org",
        "name": "Artturi N"
      }
    ],
    "name": "zsh-5.9",
    "outputsToInstall": [
      "out",
      "man"
    ],
    "platforms": [
      "i686-cygwin",
      "x86_64-cygwin",
      "x86_64-darwin",
      "i686-darwin",
      "aarch64-darwin",
      "armv7a-darwin",
      "i686-freebsd13",
      "x86_64-freebsd13",
      "x86_64-solaris",
      "aarch64-linux",
      "armv5tel-linux",
      "armv6l-linux",
      "armv7a-linux",
      "armv7l-linux",
      "i686-linux",
      "m68k-linux",
      "microblaze-linux",
      "microblazeel-linux",
      "mipsel-linux",
      "mips64el-linux",
      "powerpc64-linux",
      "powerpc64le-linux",
      "riscv32-linux",
      "riscv64-linux",
      "s390-linux",
      "s390x-linux",
      "x86_64-linux",
      "aarch64-netbsd",
      "armv6l-netbsd",
      "armv7a-netbsd",
      "armv7l-netbsd",
      "i686-netbsd",
      "m68k-netbsd",
      "mipsel-netbsd",
      "powerpc-netbsd",
      "riscv32-netbsd",
      "riscv64-netbsd",
      "x86_64-netbsd",
      "i686-openbsd",
      "x86_64-openbsd",
      "x86_64-redox"
    ],
    "position": "/nix/store/ccx3ky2h0lqfxn142lh701m19pw2wj0y-nixpkgs/nixpkgs/pkgs/shells/zsh/default.nix:103",
    "unfree": false,
    "unsupported": false
  }
}
*/
