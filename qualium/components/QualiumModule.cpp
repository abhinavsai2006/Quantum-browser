/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
#include "mozilla/ModuleUtils.h"
#include "nsIClassInfoImpl.h"
#include "QualiumSecurityService.h"

NS_GENERIC_FACTORY_CONSTRUCTOR_INIT(QualiumSecurityService, Init)
NS_DEFINE_NAMED_CID(QUALIUM_SECURITY_SERVICE_CID);

static const mozilla::Module::CIDEntry kQualiumCIDs[] = {
  { &kQUALIUM_SECURITY_SERVICE_CID, false, nullptr, QualiumSecurityServiceConstructor },
  { nullptr }
};

static const mozilla::Module::ContractIDEntry kQualiumContracts[] = {
  { QUALIUM_SECURITY_SERVICE_CONTRACTID, &kQUALIUM_SECURITY_SERVICE_CID },
  { nullptr }
};

static const mozilla::Module kQualiumModule = {
  mozilla::Module::kVersion,
  kQualiumCIDs,
  kQualiumContracts,
  nullptr,
  nullptr,
  nullptr,
  nullptr
};

NSMODULE_DEFN(QualiumSecurityModule) = &kQualiumModule;
