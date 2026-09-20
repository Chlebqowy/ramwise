#
# spec file for package ramwise
#
# Copyright (c) 2026 SUSE LLC
#
# All modifications and additions to the file contributed by third parties
# remain the property of their copyright owners, unless otherwise agreed
# upon. The license for this file, and modifications and additions to the
# file, is the same license as for the pristine package itself (unless the
# license for the pristine package is not an Open Source License, in which
# case the license is the MIT License). An "Open Source License" is a
# license that conforms to the Open Source Definition (Version 1.9)
# published by the Open Source Initiative.

# Please submit bugfixes or comments via https://bugs.opensuse.org/
#


Name:           ramwise
Version:        0.1.0
Release:        1%{?dist}
Summary:        a modern memory visualiser tui
License:        MIT
URL:            https://github.com/Duckaet/ramwise
Source0:        https://github.com/Duckaet/ramwise/releases/download/v0.1.0/ramwise-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
BuildRequires:  cargo, make
BuildArch:      x86_64

%description
ramwise is a terminal-based RAM usage visualizer that goes beyond basic memory monitoring. It provides deep memory introspection, intelligent leak detection, and beautiful visualization all in a lightweight TUI application.

%prep
%setup -q

%build
make %{?_smp_mflags}

%install
rm -rf $RPM_BUILD_ROOT 
%make_install

%files 
%license LICENSE
/usr/bin/ramwise

%changelog
* Sun Sep 13 2026 - v0.1.0
- First release
