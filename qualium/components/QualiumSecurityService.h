/* -*- Mode: C++; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
#ifndef QualiumSecurityService_h
#define QualiumSecurityService_h

#include "nsIQualiumSecurityService.h"
#include "nsCOMPtr.h"
#include "nsString.h"
#include "mozilla/Mutex.h"

#define QUALIUM_SECURITY_SERVICE_CID \
  { 0x9a2f3fac, 0x0d4a, 0x44cd, { 0x8d, 0xdd, 0x67, 0x13, 0xb3, 0x0d, 0x95, 0x74 } }

#define QUALIUM_SECURITY_SERVICE_CONTRACTID \
  "@qualium.ai/security-service;1"

class QualiumSecurityService final : public nsIQualiumSecurityService
{
public:
  NS_DECL_ISUPPORTS
  NS_DECL_NSIQUALIUMSECURITYSERVICE

  QualiumSecurityService();

  nsresult Init();

private:
  ~QualiumSecurityService() = default;

  uint32_t mPrivacyLevel;
  uint64_t mAdsBlockedCount;
  uint64_t mTrackersBlockedCount;
  mozilla::Mutex mMutex;
};

#endif // QualiumSecurityService_h
