%global debug_package %{nil}

Name:           capri-desktop
Version:        1.0.0
Release:        1%{?dist}
Summary:        Capri Desktop Mascot & Cyberpunk Ollama AI Construct

License:        MIT
URL:            https://github.com/lachydotmcg/capri-desktop
Source0:        capri-desktop-1.0.0.tar.gz
Source1:        capri-desktop.desktop
Source2:        capri-desktop.png
Source3:        LICENSE

BuildArch:      x86_64

%description
Capri Desktop is an interactive Cyberpunk construct and desktop pet for Linux.
Rebuilt natively in Rust with custom animations, interactive physics, dialogue bubbles,
and integrated local AI chat powered by Ollama.

%prep
%setup -q -c

%build
# Pre-built optimized Rust binary with embedded assets

%install
rm -rf %{buildroot}
install -d %{buildroot}%{_bindir}
install -p -m 0755 capri %{buildroot}%{_bindir}/capri
ln -s capri %{buildroot}%{_bindir}/capri-desktop

install -d %{buildroot}%{_datadir}/applications
install -p -m 0644 %{SOURCE1} %{buildroot}%{_datadir}/applications/capri-desktop.desktop

install -d %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
install -p -m 0644 %{SOURCE2} %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/capri-desktop.png

install -d %{buildroot}%{_datadir}/licenses/%{name}
install -p -m 0644 %{SOURCE3} %{buildroot}%{_datadir}/licenses/%{name}/LICENSE

%files
%license %{_datadir}/licenses/%{name}/LICENSE
%{_bindir}/capri
%{_bindir}/capri-desktop
%{_datadir}/applications/capri-desktop.desktop
%{_datadir}/icons/hicolor/scalable/apps/capri-desktop.png

%changelog
* Sun Oct 25 2026 Capri Packagers <support@capri.local> - 1.0.0-1
- Initial native Linux release rewritten in Rust
- Integrated Ollama 3B abliterated construct support
- Animated sprites and interactive dialogue speech bubble
