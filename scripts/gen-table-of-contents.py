#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import sys

import pprint

VERBOSE = False

# 1 is the title of the post
# 2 is the table of contents
# the sections start at 3
OFFSET = 3


def get_label(section_title: str) -> str:
    if not section_title:
        raise ValueError("Empty title")

    return (
        section_title.strip()
        .lower()
        .replace(" ", "-")
        .replace(":", "")
        .replace(".", "")
        .replace(",", "")
    )


if len(sys.argv) < 2:
    sys.stderr.write("Filename not provided\n")
    sys.exit(1)

filename: str = sys.argv[1]

if not os.path.isfile(filename):
    sys.stderr.write(f"File '{filename}' does not exist\n")
    sys.exit(1)

with open(filename, "r", encoding="utf-8") as f:
    lines = f.readlines()

entries = [line.strip() for line in lines if line.strip().startswith("#")]

# Remove the table of contents if it is present
entries.remove("## Table of Contents")

if VERBOSE:
    pprint.pprint(entries)

print("## Table of Contents", end="\n\n")
for e in entries:
    preffix_length = len(e) - len(e.lstrip("#"))
    assert preffix_length >= OFFSET
    num_tabs = preffix_length - OFFSET

    section_title = e.lstrip("#").strip()

    if num_tabs > 0:
        print(f"{'\t' * num_tabs} - [{section_title}]({'#' + get_label(e.lstrip('#'))})")
    else:
        print(f"- [{section_title}]({'#' + get_label(e.lstrip('#'))})")

    print("---", end="\n\n")  #
    print("<br/>")

