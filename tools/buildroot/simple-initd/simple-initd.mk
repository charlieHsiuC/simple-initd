SIMPLE_INITD_VERSION = 0.1.0
SIMPLE_INITD_SITE = 
SIMPLE_INITD_SITE_METHOD = local

$(eval $(cargo-package))

define SIMPLE_INITD_INSTALL_TARGET_CMDS
    $(INSTALL) -D -m 0755 $(@D)/target/$(RUSTC_TARGET_NAME)/release/simple-initd $(TARGET_DIR)/sbin/init
endef