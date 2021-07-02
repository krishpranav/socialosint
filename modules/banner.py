#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# imports
from modules.colors import colors
import os
def banner():
    
    banner = "SOCIAL OSINT"
    ban = banner.split("\n")
    for line in ban:
        centered = line.center(os.get_terminal_size().columns)
        print(colors.BOLD + centered + colors.end)