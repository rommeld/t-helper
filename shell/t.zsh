# Source this file from an interactive zsh, after any prompt setup.
[[ -o interactive ]] || return 0
(( ${+functions[_t_helper_precmd]} )) && return 0

if (( ${+aliases[t]} || ${+galiases[t]} || ${+functions[t]} || ${+builtins[t]} )) ||
    builtin whence -p -- t >/dev/null 2>&1; then
    builtin print -u2 -- 't-helper: t already exists. Remove or rename it before loading t-helper.'
    return 1
fi

typeset -g +x _t_helper_pending='' _t_helper_command='' _t_helper_status=''
typeset -g +x _t_helper_pending_event='' _t_helper_event=''

function _t_helper_snapshot_matches {
    # The next submission removes temporary history entries and may reuse their
    # event number. Never read a recycled event or substitute an older event.
    [[ -z $_t_helper_pending_event && -n $_t_helper_event ]] || return 1
    (( HISTCMD > _t_helper_event )) || return 1
    if (( HISTCMD > _t_helper_event + 1 )) &&
        builtin fc -lLI "$((_t_helper_event + 1))" "$((HISTCMD - 1))" >/dev/null 2>&1; then
        # Shared history can add foreign events, but not another local command.
        return 1
    fi

    # Look up exactly one local event. Reject imported entries even if their
    # text matches. Successful commands need only the event, not the text.
    local pattern='*'
    if [[ -n $_t_helper_command ]]; then
        pattern=${(b)_t_helper_command}
    fi
    builtin fc -lLI -m "$pattern" "$_t_helper_event" "$_t_helper_event" >/dev/null 2>&1
}

function _t_helper_preexec {
    # preexec can still receive leading-space commands with HIST_IGNORE_SPACE set.
    local ignore_space=${options[histignorespace]}
    emulate -L zsh
    # This also invalidates the snapshot if the previous precmd was skipped.
    if ! _t_helper_snapshot_matches; then
        _t_helper_command=''
        _t_helper_status=''
        _t_helper_event=''
    fi
    _t_helper_pending_event=$HISTCMD
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
    _t_helper_event=''
    # A prompt with no preexec can follow a parse error. Never reuse an old failure.
    if [[ -n $_t_helper_pending ]] && (( HISTCMD == _t_helper_pending_event + 1 )); then
        _t_helper_status=$exit_code
        _t_helper_event=$_t_helper_pending_event
        if (( exit_code != 0 )); then
            _t_helper_command=$_t_helper_pending
        fi
    fi
    _t_helper_pending=''
    _t_helper_pending_event=''
    return 0
}

function t {
    emulate -L zsh
    if (( $# == 1 )) && [[ $1 == -h || $1 == --help ]]; then
        builtin print -r -- 'Usage: t    Offer a hint for the previous failed command in this zsh session.'
    elif (( $# != 0 )); then
        builtin print -u2 -- 't-helper: t takes no arguments. See t --help.'
        return 2
    elif [[ -z $_t_helper_status || $_t_helper_pending_event != $HISTCMD ]]; then
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
