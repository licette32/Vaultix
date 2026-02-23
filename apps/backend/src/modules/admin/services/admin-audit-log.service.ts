import { Injectable } from '@nestjs/common';
import { InjectRepository } from '@nestjs/typeorm';
import { Repository, FindManyOptions } from 'typeorm';
import { AdminAuditLog } from '../entities/admin-audit-log.entity';

export interface CreateAuditLogDto {
  actorId: string;
  actionType: string;
  resourceType: string;
  resourceId?: string | null;
  metadata?: Record<string, any>;
}

@Injectable()
export class AdminAuditLogService {
  constructor(
    @InjectRepository(AdminAuditLog)
    private readonly auditLogRepo: Repository<AdminAuditLog>,
  ) {}

  async create(dto: CreateAuditLogDto): Promise<AdminAuditLog> {
    const log = this.auditLogRepo.create({
      ...dto,
      resourceId: dto.resourceId ?? null,
      metadata: dto.metadata ?? {},
    });
    return this.auditLogRepo.save(log);
  }

  async findAll(
    filters: {
      actorId?: string;
      actionType?: string;
      resourceType?: string;
      resourceId?: string;
      from?: Date;
      to?: Date;
      page?: number;
      pageSize?: number;
    } = {},
  ): Promise<{ data: AdminAuditLog[]; total: number }> {
    const {
      actorId,
      actionType,
      resourceType,
      resourceId,
      from,
      to,
      page = 1,
      pageSize = 20,
    } = filters;
    const where: any = {};
    if (actorId) where.actorId = actorId;
    if (actionType) where.actionType = actionType;
    if (resourceType) where.resourceType = resourceType;
    if (resourceId) where.resourceId = resourceId;
    if (from || to) {
      where.createdAt = {};
      if (from) where.createdAt['$gte'] = from;
      if (to) where.createdAt['$lte'] = to;
    }
    const [data, total] = await this.auditLogRepo.findAndCount({
      where,
      order: { createdAt: 'DESC' },
      skip: (page - 1) * pageSize,
      take: pageSize,
    } as FindManyOptions<AdminAuditLog>);
    return { data, total };
  }
}
