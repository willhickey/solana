#!/usr/bin/env sh

function semverParseInto() {
    local RE='[^0-9]*\([0-9]*\)[.]\([0-9]*\)[.]\([0-9]*\)\([0-9A-Za-z-]*\)[.]\?\([0-9]*\)'
    #MAJOR
    eval $2=`echo $1 | sed -e "s#$RE#\1#"`
    #MINOR
    eval $3=`echo $1 | sed -e "s#$RE#\2#"`
    #MINOR
    eval $4=`echo $1 | sed -e "s#$RE#\3#"`
    #PRERELEASE_STAGE
    eval $5=`echo $1 | sed -e "s#$RE#\4#"`
    #PRERELEASE_NUMBER
    eval $6=`echo $1 | sed -e "s#$RE#\5#"`
}

function semverEQ() {
    local MAJOR_A=0
    local MINOR_A=0
    local PATCH_A=0
    local PRERELEASE_STAGE_A=0
    local PRERELEASE_NUMBER_A=0

    local MAJOR_B=0
    local MINOR_B=0
    local PATCH_B=0
    local PRERELEASE_STAGE_B=0
    local PRERELEASE_NUMBER_B=0

    semverParseInto $1 MAJOR_A MINOR_A PATCH_A PRERELEASE_STAGE_A PRERELEASE_NUMBER_A
    semverParseInto $2 MAJOR_B MINOR_B PATCH_B PRERELEASE_STAGE_B PRERELEASE_NUMBER_B

    if [ $MAJOR_A -ne $MAJOR_B ]; then
        return 1
    fi

    if [ $MINOR_A -ne $MINOR_B ]; then
        return 1
    fi

    if [ $PATCH_A -ne $PATCH_B ]; then
        return 1
    fi

    if [[ "_$PRERELEASE_STAGE_A" != "_$PRERELEASE_STAGE_B" ]]; then
        return 1
    fi

    if [[ "_$PRERELEASE_NUMBER_A" != "_$PRERELEASE_NUMBER_B" ]]; then
        return 1
    fi

    return 0

}

function semverLT() {
    local MAJOR_A=0
    local MINOR_A=0
    local PATCH_A=0
    local PRERELEASE_STAGE_A=0
    local PRERELEASE_NUMBER_A=0

    local MAJOR_B=0
    local MINOR_B=0
    local PATCH_B=0
    local PRERELEASE_STAGE_B=0
    local PRERELEASE_NUMBER_B=0

    semverParseInto $1 MAJOR_A MINOR_A PATCH_A PRERELEASE_STAGE_A PRERELEASE_NUMBER_A
    semverParseInto $2 MAJOR_B MINOR_B PATCH_B PRERELEASE_STAGE_B PRERELEASE_NUMBER_B

    if [ $MAJOR_A -lt $MAJOR_B ]; then
        return 0
    fi

    if [[ $MAJOR_A -le $MAJOR_B  && $MINOR_A -lt $MINOR_B ]]; then
        return 0
    fi

    if [[ $MAJOR_A -le $MAJOR_B  && $MINOR_A -le $MINOR_B && $PATCH_A -lt $PATCH_B ]]; then
        return 0
    fi

    if [[ "_$PRERELEASE_STAGE_A"  == "_" ]] && [[ "_$PRERELEASE_STAGE_B"  == "_" ]] ; then
        return 1
    fi
    if [[ "_$PRERELEASE_STAGE_A"  == "_" ]] && [[ "_$PRERELEASE_STAGE_B"  != "_" ]] ; then
        return 1
    fi
    if [[ "_$PRERELEASE_STAGE_A"  != "_" ]] && [[ "_$PRERELEASE_STAGE_B"  == "_" ]] ; then
        return 0
    fi

    if [[ "_$PRERELEASE_STAGE_A" < "_$PRERELEASE_STAGE_B" ]]; then
        return 0
    fi

    if [[ "$PRERELEASE_STAGE_A" == "$PRERELEASE_STAGE_B" ]]; then
        if [[ "_$PRERELEASE_NUMBER_A"  == "_" ]] && [[ "_$PRERELEASE_NUMBER_B"  != "_" ]] ; then
            return 1
        fi
        if [[ "_$PRERELEASE_NUMBER_A"  != "_" ]] && [[ "_$PRERELEASE_NUMBER_B"  == "_" ]] ; then
            return 0
        fi
        if [[ $PRERELEASE_NUMBER_A -lt $PRERELEASE_NUMBER_B ]]; then
            return 0
        fi
    fi

    return 1

}

function semverGT() {
    semverEQ $1 $2
    local EQ=$?

    semverLT $1 $2
    local LT=$?

    if [ $EQ -ne 0 ] && [ $LT -ne 0 ]; then
        return 0
    else
        return 1
    fi
}

if [ "___semver.sh" == "___`basename $0`" ]; then

MAJOR=0
MINOR=0
PATCH=0
PRERELEASE_STAGE=""
PRERELEASE_NUMBER=""

semverParseInto $1 MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$1 -> M: $MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER"

semverParseInto $2 MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$2 -> M: $MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER"

semverEQ $1 $2
echo "$1 == $2 -> $?."

semverLT $1 $2
echo "$1 < $2 -> $?."

semverGT $1 $2
echo "$1 > $2 -> $?."

fi
