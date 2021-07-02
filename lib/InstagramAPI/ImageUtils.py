#!/usr/bin/env python3
# -*- coding: utf-8 -*-

# imports
import struct
import imghdr

def getImageSize(fnmae):
    with open(fnmae, 'rb') as fhandle:
        head = fhandle.read(24)
        if len(head) != 24:
            raise RuntimeError("Invalid Header")
        if imghdr.what(fnmae) == 'png':
            check = struct.unpack('>i', head[4:8])[0]
            if check != 0x0d0a1a0a:
                raise RuntimeError("PNG: Invalid Check")
            width, height = struct.unpack('>ii', head[16:24])
            