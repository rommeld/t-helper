# Source this file from an interactive zsh, after any prompt setup.
[[ -o interactive ]] || return 0
(( ${+functions[_t_helper_precmd]} )) && return 0

if (( ${+aliases[t]} || ${+galiases[t]} || ${+functions[t]} || ${+builtins[t]} )) ||
    builtin whence -p -- t >/dev/null 2>&1; then
    builtin print -u2 -- 't-helper: t already exists. Remove or rename it before loading t-helper.'
    return 1
fi

typeset -g +x _t_helper_pending='' _t_helper_command='' _t_helper_status=''

function _t_helper_preexec {
    # preexec can still receive leading-space commands with HIST_IGNORE_SPACE set.
    local ignore_space=${options[histignorespace]}
    emulate -L zsh
    _t_helper_pending=''
    if [[ -n ${1-} && ( $ignore_space != on || $1 != ' '* ) ]]; then
        _t_helper_pending=$1
    fi
    return 0
}

function _t_helper_precmd {
    local exit_code=$?
    emulate -L zsh
    _t_helper_command=''
    _t_helper_status=''
    # A prompt with no preexec can follow a parse error. Never reuse an old failure.
    if [[ -n $_t_helper_pending ]]; then
        _t_helper_status=$exit_code
        if (( exit_code != 0 )); then
            _t_helper_command=$_t_helper_pending
        fi
    fi
    _t_helper_pending=''
    return 0
}

function t {
    emulate -L zsh
    if (( $# == 1 )) && [[ $1 == -h || $1 == --help ]]; then
        builtin print -r -- 'Usage: t    Offer a hint for the previous failed command in this zsh session.'
    elif (( $# != 0 )); then
        builtin print -u2 -- 't-helper: t takes no arguments. See t --help.'
        return 2
    elif [[ -z $_t_helper_status ]]; then
        builtin print -r -- 'No previous command is available to diagnose.'
    elif [[ $_t_helper_status == 0 ]]; then
        builtin print -r -- 'The previous command succeeded.'
    elif ! builtin whence -p -- t-helper >/dev/null 2>&1; then
        builtin print -u2 -- 't-helper: binary not found on PATH. Run cargo install --locked --path . from the t-helper checkout.'
        return 1
    else
        builtin printf '%s\0%s\0' "$_t_helper_status" "$_t_helper_command" | command 't-helper' diagnose
    fi
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec _t_helper_preexec
add-zsh-hook precmd _t_helper_precmd
