### **Week 2: GhostBridge Integration**
**🎯 Objective**: Connect ZNS to GhostChain via gRPC bridge

**Key Tasks:**
- [ ] **Implement gRPC ZNS Interface** (Spec in `ZNS_GRPC_INTERFACE.md`)
  ```protobuf
  service ZNSService {
    rpc ResolveDomain(ZNSResolveRequest) returns (ZNSResolveResponse);
    rpc RegisterDomain(ZNSRegisterRequest) returns (ZNSRegisterResponse);
    rpc SubscribeDomainChanges(DomainSubscription) returns (stream DomainChangeEvent);
  }
  ```

- [ ] **GhostChain ZNS Integration Module**
  ```rust
  // src/zns_integration.rs
  pub struct ZnsIntegration {
      bridge_client: GhostBridgeClient,
      domain_storage: DomainStorage,
  }
  ```

- [ ] **Domain Query Bridge**
  - ZNS queries blockchain for domain ownership
  - Smart contract state synchronization
  - Real-time domain change notifications

- [ ] **End-to-End Testing**
  - Deploy test contract with domain
  - Query domain via ZNS
  - Verify ownership and DNS records

**Success Criteria:**
- ZNS communicates with GhostChain via gRPC
- Domain queries return blockchain-verified results
- Real-time domain updates propagate within 1 second


