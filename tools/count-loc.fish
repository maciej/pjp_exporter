#!/usr/bin/env fish

rg -v '^\w*$' -g '**/*.rs' | wc -l

