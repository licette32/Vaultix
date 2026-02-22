import { Controller, Post, Get, Delete, Body, Param, Req, UseGuards } from '@nestjs/common';
import { WebhookService } from '../../services/webhook/webhook.service';
import { WebhookEvent } from '../../types/webhook/webhook.types';
import { AuthGuard } from '../auth/middleware/auth.guard';

class CreateWebhookDto {
  url: string;
  secret: string;
  events: WebhookEvent[];
}

@Controller('webhooks')
@UseGuards(AuthGuard)
export class WebhookController {
  constructor(private readonly webhookService: WebhookService) {}

  @Post()
  async create(@Req() req, @Body() dto: CreateWebhookDto) {
    // TODO: Rate limit per user
    return this.webhookService.createWebhook(req.user.id, dto.url, dto.secret, dto.events);
  }

  @Get()
  async list(@Req() req) {
    return this.webhookService.getUserWebhooks(req.user.id);
  }

  @Delete(':id')
  async remove(@Req() req, @Param('id') id: string) {
    await this.webhookService.deleteWebhook(req.user.id, id);
    return { success: true };
  }
}
