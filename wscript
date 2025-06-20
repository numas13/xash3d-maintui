#! /usr/bin/env python
# encoding: utf-8
# numas13, 2025

from waflib import Logs, Configure, Utils
import os
import sys

top = '.'

def options(opt):
	pass

def rust_triple(conf):
	if conf.env.DEST_CPU == 'x86_64':
		arch = 'x86_64'
	elif conf.env.DEST_CPU == 'x86':
		arch = 'i686'
	elif conf.env.DEST_CPU == 'riscv':
		arch = 'riscv64gc'
	elif conf.env.DEST_CPU == 'thumb':
		arch = 'thumbv7neon'
	elif conf.env.DEST_CPU == 'powerpc':
		# this is incorrect, but we don't know DEST endianness here?
		# we don't support big endian at this point in the engine yet
		arch = 'powerpc64le'
	else:
		arch = conf.env.DEST_CPU

	vendor = 'unknown'
	if conf.env.DEST_OS == 'linux':
		if conf.env.DEST_CPU == 'thumb':
			os = 'linux-gnueabihf'
		else:
			os = 'linux-gnu'
	elif conf.env.DEST_OS == 'win32':
		vendor = 'pc'
		if conf.env.COMPILER_CC == 'msvc':
			os = 'windows-msvc'
		else:
			os = 'windows-gnu'
	elif conf.env.DEST_OS == 'android':
		vendor = ''
		os = 'linux-android'
	elif conf.env.DEST_OS == 'darwin':
		vendor = 'apple'
		os = 'darwin'
	else:
		conf.fatal('unexpected DEST_OS ' + conf.env.DEST_OS)

	triple = arch
	if vendor != '':
		triple += '-' + vendor
	triple += '-' + os

	return triple

def configure(conf):
	conf.find_program('cargo')
	cargo = conf.env.CARGO

	opts = ['--manifest-path=%s' % conf.path.find_node('Cargo.toml')]

	triple = rust_triple(conf)
	conf.start_msg('Cargo target triple')
	conf.end_msg(triple)
	opts += ['--target=%s' % triple]

	linker = conf.env.LINK_CC[0]
	if os.sep == '\\':
		linker = linker.replace(os.sep, '/')

	opts += ['--config=target.%s.linker="%s"' % (triple, linker)]

	conf.start_msg('Cargo fetch dependencies')
	status = conf.exec_command(cargo + ['fetch'] + opts)
	if status != 0:
		conf.end_msg('exit %d' % status, color='RED')
		conf.fatal('failed to fetch Rust dependencies')
	conf.end_msg('yes')

	if conf.options.BUILD_TYPE == 'humanrights':
		opts += ['--release']
		build_type = 'release'
	else:
		build_type = 'debug'

	conf.start_msg('Cargo build type')
	conf.end_msg(build_type)

	features = []

	if features:
		features = ','.join(features)
		opts += ['--features', features]
		conf.start_msg('Cargo build features')
		conf.end_msg(features)

	lib = conf.env.cshlib_PATTERN % 'menu'
	conf.env.MAINTUI_DIST_NAME = conf.env.cshlib_PATTERN % 'menu_tui'
	conf.env.MAINTUI_CARGO = cargo
	conf.env.MAINTUI_CARGO_OPTS = opts
	conf.env.MAINTUI_TARGET = os.path.join('target', triple, build_type, lib)

def build(bld):
	rule = bld.env.MAINTUI_CARGO + ['build'] + bld.env.MAINTUI_CARGO_OPTS
	rule += ['--target-dir=%s' % os.path.join(bld.out_dir, '3rdparty', 'maintui', 'target')]

	def proc(task):
		task.exec_command(rule)

	bld(name='maintui', rule=proc, target=bld.env.MAINTUI_TARGET, always=True)

	dest = os.path.join(bld.env.LIBDIR, bld.env.MAINTUI_DIST_NAME)
	bld.install_as(dest, bld.env.MAINTUI_TARGET, chmod=0o0755)
