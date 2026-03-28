#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    mod detect {
        use super::*;

        #[test]
        fn detects_zsh() {
            assert_eq!(detect("/bin/zsh"), Some(ShellKind::Zsh));
        }

        #[test]
        fn detects_bash() {
            assert_eq!(detect("/usr/bin/bash"), Some(ShellKind::Bash));
        }

        #[test]
        fn detects_fish() {
            assert_eq!(detect("/usr/local/bin/fish"), Some(ShellKind::Fish));
        }
    }

    mod unknown {
        use super::*;

        #[test]
        fn unknown_shell_returns_none() {
            assert_eq!(detect("/bin/csh"), None);
        }

        #[test]
        fn empty_string_returns_none() {
            assert_eq!(detect(""), None);
        }
    }

    mod rc_paths {
        use super::*;

        #[test]
        fn zsh_rc_path() {
            let home = Path::new("/home/user");
            assert_eq!(rc_file_path(ShellKind::Zsh, home), home.join(".zshrc"));
        }

        #[test]
        fn bash_rc_path() {
            let home = Path::new("/home/user");
            assert_eq!(rc_file_path(ShellKind::Bash, home), home.join(".bashrc"));
        }

        #[test]
        fn fish_rc_path() {
            let home = Path::new("/home/user");
            assert_eq!(
                rc_file_path(ShellKind::Fish, home),
                home.join(".config/fish/config.fish")
            );
        }
    }

    mod completion_paths {
        use super::*;

        #[test]
        fn zsh_completion_path() {
            let home = Path::new("/home/user");
            assert_eq!(
                completion_file_path(ShellKind::Zsh, home),
                home.join(".zfunc/_ecc")
            );
        }

        #[test]
        fn bash_completion_path() {
            let home = Path::new("/home/user");
            assert_eq!(
                completion_file_path(ShellKind::Bash, home),
                home.join(".local/share/bash-completion/completions/ecc")
            );
        }

        #[test]
        fn fish_completion_path() {
            let home = Path::new("/home/user");
            assert_eq!(
                completion_file_path(ShellKind::Fish, home),
                home.join(".config/fish/completions/ecc.fish")
            );
        }
    }

    mod fish_no_source {
        use super::*;

        #[test]
        fn fish_completion_source_line_is_none() {
            assert_eq!(completion_source_line(ShellKind::Fish), None);
        }

        #[test]
        fn zsh_has_source_line() {
            assert!(completion_source_line(ShellKind::Zsh).is_some());
        }

        #[test]
        fn bash_has_source_line() {
            assert!(completion_source_line(ShellKind::Bash).is_some());
        }
    }

    mod block_assembly {
        use super::*;

        #[test]
        fn zsh_block_has_path_and_compinit() {
            let block = build_rc_block(ShellKind::Zsh);
            assert_eq!(block.len(), 2);
            assert!(block[0].contains("PATH"));
            assert!(block[1].contains("compinit"));
        }

        #[test]
        fn fish_block_has_only_fish_add_path() {
            let block = build_rc_block(ShellKind::Fish);
            assert_eq!(block.len(), 1);
            assert!(block[0].contains("fish_add_path"));
        }

        #[test]
        fn bash_block_has_path_and_source() {
            let block = build_rc_block(ShellKind::Bash);
            assert_eq!(block.len(), 2);
            assert!(block[0].contains("PATH"));
            assert!(block[1].contains("source"));
        }
    }
}
