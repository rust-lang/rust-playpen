#!/bin/false
# This file should be source-d rather than exec-ed

coloredtest=0
while : ; do
    case "$1" in
        --evaluatesh=*|--compilesh=*)
            case "$1" in
                --evaluatesh=debug|--compilesh=debug)
                    export RUST_BACKTRACE=1
                    debugme=1
                    ;;
                --evaluatesh=coloredtest)
                    coloredtest=1
                    ;;
            esac
            shift
            ;;
        *)
            break
    esac
done
