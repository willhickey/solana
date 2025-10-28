#!/usr/bin/env bash

. ./semver.sh

semverTest() {
local A=R1.3.2
local B=R2.3.2
local C=R1.4.2
local D=R1.3.3
local E=R1.3.2a
local F=R1.3.2b
local G=R1.2.3
local H=R1.2.3-alpha
local I=R1.2.3-beta.1
local J=R1.2.3-alpha.1
local K=R1.2.3-alpha.2
local L=R1.2.3-alpha.11




local MAJOR=0
local MINOR=0
local PATCH=0
local PRERELEASE_STAGE=""
local PRERELEASE_NUMBER=""

echo "Parsing"
semverParseInto $A MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$A -> M:$MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER. Expect M:1 m:3 p:2 ps: pn:"
semverParseInto $E MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$E -> M:$MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER. Expect M:1 m:3 p:2 ps:a pn:"
semverParseInto $H MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$H -> M:$MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER. Expect M:1 m:2 p:3 ps:-alpha pn:"
semverParseInto $J MAJOR MINOR PATCH PRERELEASE_STAGE PRERELEASE_NUMBER
echo "$J -> M:$MAJOR m:$MINOR p:$PATCH ps:$PRERELEASE_STAGE pn:$PRERELEASE_NUMBER. Expect M:1 m:2 p:3 ps:-alpha pn:1"

echo ""

echo "Equality comparisons"
semverEQ $A $A
echo "$A == $A -> $?. Expect 0."

semverLT $A $A
echo "$A < $A -> $?. Expect 1."

semverGT $A $A
echo "$A > $A -> $?. Expect 1."
echo ""

echo "Major number comparisons"
semverEQ $A $B
echo "$A == $B -> $?. Expect 1."

semverLT $A $B
echo "$A < $B -> $?. Expect 0."

semverGT $A $B
echo "$A > $B -> $?. Expect 1."

semverEQ $B $A
echo "$B == $A -> $?. Expect 1."

semverLT $B $A
echo "$B < $A -> $?. Expect 1."

semverGT $B $A
echo "$B > $A -> $?. Expect 0."
echo ""

echo "Minor number comparisons"
semverEQ $A $C
echo "$A == $C -> $?. Expect 1."

semverLT $A $C
echo "$A < $C -> $?. Expect 0."

semverGT $A $C
echo "$A > $C -> $?. Expect 1."

semverEQ $C $A
echo "$C == $A -> $?. Expect 1."

semverLT $C $A
echo "$C < $A -> $?. Expect 1."

semverGT $C $A
echo "$C > $A -> $?. Expect 0."
echo ""

echo "patch number comparisons"
semverEQ $A $D
echo "$A == $D -> $?. Expect 1."

semverLT $A $D
echo "$A < $D -> $?. Expect 0."

semverGT $A $D
echo "$A > $D -> $?. Expect 1."

semverEQ $D $A
echo "$D == $A -> $?. Expect 1."

semverLT $D $A
echo "$D < $A -> $?. Expect 1."

semverGT $D $A
echo "$D > $A -> $?. Expect 0."
echo ""

echo "special section vs no special comparisons"
semverEQ $A $E
echo "$A == $E -> $?. Expect 1."

semverLT $A $E
echo "$A < $E -> $?. Expect 1."

semverGT $A $E
echo "$A > $E -> $?. Expect 0."

semverEQ $E $A
echo "$E == $A -> $?. Expect 1."

semverLT $E $A
echo "$E < $A -> $?. Expect 0."

semverGT $E $A
echo "$E > $A -> $?. Expect 1."
echo ""

echo "special section vs special comparisons"
semverEQ $E $F
echo "$E == $F -> $?. Expect 1."

semverLT $E $F
echo "$E < $F -> $?. Expect 0."

semverGT $E $F
echo "$E > $F -> $?. Expect 1."

semverEQ $F $E
echo "$F == $E -> $?. Expect 1."

semverLT $F $E
echo "$F < $E -> $?. Expect 1."

semverGT $F $E
echo "$F > $E -> $?. Expect 0."
echo ""

echo "Minor and patch number comparisons"
semverEQ $A $G
echo "$A == $G -> $?. Expect 1."

semverLT $A $G
echo "$A < $G -> $?. Expect 1."

semverGT $A $G
echo "$A > $G -> $?. Expect 0."

semverEQ $G $A
echo "$G == $A -> $?. Expect 1."

semverLT $G $A
echo "$G < $A -> $?. Expect 0."

semverGT $G $A
echo "$G > $A -> $?. Expect 1."
echo ""

# local H=R1.2.3-alpha
# local I=R1.2.3-beta.1
# local J=R1.2.3-alpha.1
# local K=R1.2.3-alpha.2
# local L=R1.2.3-alpha.11

# HJ
# JK
# JL
# IK
# IL


echo "Different prerelease stages"
semverEQ $H $I
echo "$H == $I -> $?. Expect 1."

semverLT $H $I
echo "$H < $I -> $?. Expect 0."

semverGT $H $I
echo "$H > $I -> $?. Expect 1."

semverEQ $I $H
echo "$I == $H -> $?. Expect 1."

semverLT $I $H
echo "$I < $H -> $?. Expect 1."

semverGT $I $H
echo "$I > $H -> $?. Expect 0."
echo ""

# TODO this one reveals a bug
echo "Same prerelease stage, one with number and one without"
semverEQ $H $J
echo "$H == $J -> $?. Expect 1."

semverLT $H $J
echo "$H < $J -> $?. Expect 1."

semverGT $H $J
echo "$H > $J -> $?. Expect 0."

semverEQ $J $H
echo "$J == $H -> $?. Expect 1."

semverLT $J $H
echo "$J < $H -> $?. Expect 0."

semverGT $J $H
echo "$J > $H -> $?. Expect 1."
echo ""

echo "Same except for prerelease number"
semverEQ $J $K
echo "$J == $K -> $?. Expect 1."

semverLT $J $K
echo "$J < $K -> $?. Expect 0."

semverGT $J $K
echo "$J > $K -> $?. Expect 1."

semverEQ $K $J
echo "$K == $J -> $?. Expect 1."

semverLT $K $J
echo "$K < $J -> $?. Expect 1."

semverGT $K $J
echo "$K > $J -> $?. Expect 0."
echo ""

echo "Same except for prerelease number"
semverEQ $J $L
echo "$J == $L -> $?. Expect 1."

semverLT $J $L
echo "$J < $L -> $?. Expect 0."

semverGT $J $L
echo "$J > $L -> $?. Expect 1."

semverEQ $L $J
echo "$L == $J -> $?. Expect 1."

semverLT $L $J
echo "$L < $J -> $?. Expect 1."

semverGT $L $J
echo "$L > $J -> $?. Expect 0."
echo ""

echo "Same except for prerelease number"
semverEQ $I $K
echo "$I == $K -> $?. Expect 1."

semverLT $I $K
echo "$I < $K -> $?. Expect 1."

semverGT $I $K
echo "$I > $K -> $?. Expect 0."

semverEQ $K $I
echo "$K == $I -> $?. Expect 1."

semverLT $K $I
echo "$K < $I -> $?. Expect 0."

semverGT $K $I
echo "$K > $I -> $?. Expect 1."
echo ""

echo "Same except for prerelease number"
semverEQ $I $L
echo "$I == $L -> $?. Expect 1."

semverLT $I $L
echo "$I < $L -> $?. Expect 1."

semverGT $I $L
echo "$I > $L -> $?. Expect 0."

semverEQ $L $I
echo "$L == $I -> $?. Expect 1."

semverLT $L $I
echo "$L < $I -> $?. Expect 0."

semverGT $L $I
echo "$L > $I -> $?. Expect 1."
echo ""

}

semverTest
