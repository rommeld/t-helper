# Source this file from an interactive fish session or config.fish.
status is-interactive; or return 0
functions --query __t_helper_fish_postexec; and return 0

if type --query t
    printf '%s\n' 't-helper: t already exists. Remove or rename it before loading t-helper.' >&2
    return 1
end

function __t_helper_fish_is_private
    test -n "$fish_private_mode"; or begin
        set --query fish_history; and test -z "$fish_history"
    end
end

function __t_helper_fish_clear --on-event fish_posterror --on-event fish_cancel \
        --on-variable fish_private_mode --on-variable fish_history
    set --erase --global __t_helper_fish_before __t_helper_fish_completed
end

function __t_helper_fish_preexec --on-event fish_preexec
    set --global --unexport __t_helper_fish_before $status_generation
    if __t_helper_fish_is_private
        __t_helper_fish_clear
    end
end

function __t_helper_fish_postexec --on-event fish_postexec
    set --local outcome $status $status_generation
    set --local before $__t_helper_fish_before
    __t_helper_fish_clear

    # A comment can carry the previous status without producing a new one.
    if not set --query before[1]; or test "$before" = "$outcome[2]"
        return 0
    end
    if __t_helper_fish_is_private; or string match --quiet ' *' -- "$argv[1]"
        return 0
    end

    # Store status, generation, and (only for a failure) the submitted line.
    if test "$outcome[1]" = 0
        set --global --unexport __t_helper_fish_completed $outcome
    else
        set --global --unexport __t_helper_fish_completed $outcome "$argv[1]"
    end
end

function t --description 'Offer a hint for the previous failed command'
    set --local generation $status_generation
    if test (count $argv) -eq 1; and contains -- "$argv[1]" -h --help
        printf '%s\n' 'Usage: t    Offer a hint for the previous failed command in this fish session.'
    else if set --query argv[1]
        printf '%s\n' 't-helper: t takes no arguments. See t --help.' >&2
        return 2
    else if not set --query __t_helper_fish_completed[2]; or \
            test "$__t_helper_fish_completed[2]" != "$generation"; or __t_helper_fish_is_private
        printf '%s\n' 'No previous command is available to diagnose.'
    else if test "$__t_helper_fish_completed[1]" = 0
        printf '%s\n' 'The previous command succeeded.'
    else if not command --query t-helper
        printf '%s\n' 't-helper: binary not found on PATH. Run cargo install --locked --path . from the t-helper checkout.' >&2
        return 1
    else
        builtin printf '%s\0%s\0' "$__t_helper_fish_completed[1]" "$__t_helper_fish_completed[3]" | command t-helper diagnose
    end
end
